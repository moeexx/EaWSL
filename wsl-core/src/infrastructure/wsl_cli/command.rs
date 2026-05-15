use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::sync::mpsc::{self, Sender, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::domain::error::map_wsl_error_with_output;
use crate::infrastructure::wsl_cli::decoder::{decode_command_output, DecodedCommandOutput};
use crate::infrastructure::wsl_cli::parser::error::{
    extract_wsl_error_code, looks_like_invalid_argument_output,
};
use crate::infrastructure::wsl_cli::runner::WslCommandRunner;
use crate::{ExportFormat, ProgressEvent, WslCommandContext, WslError};

const TIMEOUT_TRIGGERED: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WslCommandPolicy {
    pub timeout: Duration,
    pub cooldown: Duration,
    pub max_retries: usize,
}

pub(crate) const WSL_QUERY_POLICY: WslCommandPolicy = WslCommandPolicy {
    timeout: Duration::from_secs(5),
    cooldown: Duration::from_millis(500),
    max_retries: 1,
};

pub(crate) const WSL_ACTION_POLICY: WslCommandPolicy = WslCommandPolicy {
    timeout: Duration::from_secs(10),
    cooldown: Duration::from_millis(0),
    max_retries: 0,
};

pub(crate) async fn run_no_output_command_with_policy<R>(
    runner: &R,
    context: WslCommandContext,
    args: &[&str],
    policy: WslCommandPolicy,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    run_capture_with_policy(runner, context, args, policy)
        .await
        .map(|_| ())
}

pub(crate) fn export_format_arg(format: &ExportFormat) -> &'static str {
    match format {
        ExportFormat::Tar => "tar",
        ExportFormat::TarGz => "tar.gz",
        ExportFormat::TarXz => "tar.xz",
        ExportFormat::Vhd => "vhd",
    }
}

pub(crate) fn spawn_progress_forwarder(
    tx: Sender<ProgressEvent>,
) -> (UnboundedSender<ProgressEvent>, tokio::task::JoinHandle<()>) {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let forwarder = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let _ = tx.send(event).await;
        }
    });

    (event_tx, forwarder)
}

pub(crate) async fn run_capture_with_policy<R>(
    runner: &R,
    context: WslCommandContext,
    args: &[&str],
    policy: WslCommandPolicy,
) -> Result<DecodedCommandOutput, WslError>
where
    R: WslCommandRunner + Sync,
{
    let mut attempt = 0;

    loop {
        match run_capture_once_with_policy(runner, context, args, policy).await {
            Err(WslError::WslCommandTimedOut { .. }) if attempt < policy.max_retries => {
                attempt += 1;

                if !policy.cooldown.is_zero() {
                    tokio::time::sleep(policy.cooldown).await;
                }
            }
            result => return result,
        }
    }
}

async fn run_capture_once_with_policy<R>(
    runner: &R,
    context: WslCommandContext,
    args: &[&str],
    policy: WslCommandPolicy,
) -> Result<DecodedCommandOutput, WslError>
where
    R: WslCommandRunner + Sync,
{
    let timeout_triggered = Arc::new(AtomicU8::new(0));
    let cancel_token = CancellationToken::new();
    let timeout_flag = Arc::clone(&timeout_triggered);
    let timeout_cancel = cancel_token.clone();
    let timeout = policy.timeout;
    let timeout_task = tokio::spawn(async move {
        tokio::time::sleep(timeout).await;
        timeout_flag.store(TIMEOUT_TRIGGERED, Ordering::SeqCst);
        timeout_cancel.cancel();
    });

    let mut on_started = || {};
    let mut on_stdout = |_chunk: &[u8]| {};
    let mut on_stderr = |_chunk: &[u8]| {};

    let result = run_command(
        runner,
        args,
        Some(cancel_token),
        &mut on_started,
        &mut on_stdout,
        &mut on_stderr,
    )
    .await;

    timeout_task.abort();
    let _ = timeout_task.await;

    let decoded = match result {
        Ok(decoded) => decoded,
        Err(WslError::Cancelled)
            if timeout_triggered.load(Ordering::SeqCst) == TIMEOUT_TRIGGERED =>
        {
            return Err(WslError::WslCommandTimedOut { context });
        }
        Err(err) => return Err(err),
    };

    interpret_command_result(context, decoded)
}

pub(crate) async fn run_command<R, FStarted, FStdout, FStderr>(
    runner: &R,
    args: &[&str],
    cancel_token: Option<tokio_util::sync::CancellationToken>,
    on_started: FStarted,
    on_stdout: FStdout,
    on_stderr: FStderr,
) -> Result<DecodedCommandOutput, WslError>
where
    R: WslCommandRunner + Sync,
    FStarted: FnMut() + Send,
    FStdout: FnMut(&[u8]) + Send,
    FStderr: FnMut(&[u8]) + Send,
{
    let output = runner
        .run(args, cancel_token, on_started, on_stdout, on_stderr)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::Interrupted {
                WslError::Cancelled
            } else {
                WslError::ProcessFailed(error)
            }
        })?;
    Ok(decode_command_output(output))
}

pub(crate) fn interpret_command_result(
    context: WslCommandContext,
    decoded: DecodedCommandOutput,
) -> Result<DecodedCommandOutput, WslError> {
    match decoded.status_code {
        Some(0) => Ok(decoded),
        Some(status_code) => {
            let merged_output = decoded.merged_output.clone();
            if let Some(code) = extract_wsl_error_code(&merged_output) {
                return Err(map_wsl_error_with_output(context, &code, merged_output));
            }

            if looks_like_invalid_argument_output(&merged_output) {
                return Err(WslError::InvalidArgument {
                    context,
                    raw_output: merged_output,
                });
            }

            Err(WslError::UnknownWslError {
                context,
                code: format!("exit-status:{status_code}"),
            })
        }
        None => Err(WslError::ProcessKilled),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use super::{interpret_command_result, run_capture_with_policy, run_command, WslCommandPolicy};
    use crate::infrastructure::wsl_cli::decoder::{decode_command_output, decode_stderr};
    use crate::infrastructure::wsl_cli::runner::CommandOutput;
    use crate::infrastructure::wsl_cli::test_support::encode_utf16le;
    use crate::{WslCommandContext, WslError};

    #[test]
    fn interpret_command_result_extracts_invalid_argument_from_stdout() {
        let decoded = decode_command_output(CommandOutput {
            status_code: Some(-1),
            stdout: encode_utf16le(
                "请使用“wsl.exe --help”获取受支持的参数列表。错误代码: Wsl/E_INVALIDARG\r\n",
            ),
            stderr: Vec::new(),
        });

        let err = interpret_command_result(WslCommandContext::Version, decoded)
            .expect_err("invalid argument should be extracted from stdout");
        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::Version);
                assert!(raw_output.contains("Wsl/E_INVALIDARG"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn interpret_command_result_maps_missing_status_to_process_killed() {
        let decoded = decode_command_output(CommandOutput {
            status_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

        let err = interpret_command_result(WslCommandContext::Shutdown, decoded)
            .expect_err("missing status should map to process killed");
        assert!(matches!(err, WslError::ProcessKilled));
    }

    #[test]
    fn interpret_command_result_maps_unknown_nonzero_status() {
        let decoded = decode_command_output(CommandOutput {
            status_code: Some(42),
            stdout: b"unexpected failure".to_vec(),
            stderr: Vec::new(),
        });

        let err = interpret_command_result(WslCommandContext::Shutdown, decoded)
            .expect_err("unknown nonzero status should be preserved");
        assert!(matches!(
            err,
            WslError::UnknownWslError {
                context: WslCommandContext::Shutdown,
                code
            } if code == "exit-status:42"
        ));
    }

    #[tokio::test]
    async fn run_command_maps_interrupted_processes_to_cancelled() {
        struct InterruptingRunner;

        impl crate::infrastructure::wsl_cli::runner::WslCommandRunner for InterruptingRunner {
            fn run<'a, FStarted, FStdout, FStderr>(
                &'a self,
                _args: &'a [&'a str],
                _cancel_token: Option<tokio_util::sync::CancellationToken>,
                _on_started: FStarted,
                _on_stdout: FStdout,
                _on_stderr: FStderr,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = io::Result<CommandOutput>> + Send + 'a>,
            >
            where
                FStarted: FnMut() + Send + 'a,
                FStdout: FnMut(&[u8]) + Send + 'a,
                FStderr: FnMut(&[u8]) + Send + 'a,
            {
                Box::pin(async {
                    Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "operation cancelled",
                    ))
                })
            }
        }

        let err = run_command(
            &InterruptingRunner,
            &["--version"],
            None,
            || {},
            |_chunk| {},
            |_chunk| {},
        )
        .await
        .expect_err("interrupted run should map to cancelled");

        assert!(matches!(err, WslError::Cancelled));
    }

    #[test]
    fn stderr_ascii_output_does_not_decode_as_utf16() {
        let decoded = decode_stderr(b"e2fsck 1.46.5 (30-Dec-2021)\nresize2fs 1.46.5");
        assert!(decoded.contains("e2fsck 1.46.5"));
        assert!(decoded.contains("resize2fs"));
    }

    #[tokio::test]
    async fn run_capture_with_policy_maps_timeout_to_structured_error() {
        struct WaitingRunner;

        impl crate::infrastructure::wsl_cli::runner::WslCommandRunner for WaitingRunner {
            fn run<'a, FStarted, FStdout, FStderr>(
                &'a self,
                _args: &'a [&'a str],
                cancel_token: Option<tokio_util::sync::CancellationToken>,
                _on_started: FStarted,
                _on_stdout: FStdout,
                _on_stderr: FStderr,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = io::Result<CommandOutput>> + Send + 'a>,
            >
            where
                FStarted: FnMut() + Send + 'a,
                FStdout: FnMut(&[u8]) + Send + 'a,
                FStderr: FnMut(&[u8]) + Send + 'a,
            {
                Box::pin(async move {
                    if let Some(token) = cancel_token {
                        token.cancelled().await;
                    }

                    Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "operation cancelled",
                    ))
                })
            }
        }

        let err = run_capture_with_policy(
            &WaitingRunner,
            WslCommandContext::ListVerbose,
            &["--list", "--verbose"],
            WslCommandPolicy {
                timeout: Duration::from_millis(10),
                cooldown: Duration::from_millis(0),
                max_retries: 0,
            },
        )
        .await
        .expect_err("timeout should be mapped");

        assert!(matches!(
            err,
            WslError::WslCommandTimedOut {
                context: WslCommandContext::ListVerbose
            }
        ));
    }

    #[tokio::test]
    async fn run_capture_with_policy_retries_only_bounded_times() {
        struct RetryRunner {
            attempts: Arc<AtomicUsize>,
        }

        impl crate::infrastructure::wsl_cli::runner::WslCommandRunner for RetryRunner {
            fn run<'a, FStarted, FStdout, FStderr>(
                &'a self,
                _args: &'a [&'a str],
                cancel_token: Option<tokio_util::sync::CancellationToken>,
                _on_started: FStarted,
                _on_stdout: FStdout,
                _on_stderr: FStderr,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = io::Result<CommandOutput>> + Send + 'a>,
            >
            where
                FStarted: FnMut() + Send + 'a,
                FStdout: FnMut(&[u8]) + Send + 'a,
                FStderr: FnMut(&[u8]) + Send + 'a,
            {
                let attempts = Arc::clone(&self.attempts);

                Box::pin(async move {
                    attempts.fetch_add(1, Ordering::SeqCst);

                    if let Some(token) = cancel_token {
                        token.cancelled().await;
                    }

                    Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "operation cancelled",
                    ))
                })
            }
        }

        let attempts = Arc::new(AtomicUsize::new(0));
        let err = run_capture_with_policy(
            &RetryRunner {
                attempts: Arc::clone(&attempts),
            },
            WslCommandContext::ListVerbose,
            &["--list", "--verbose"],
            WslCommandPolicy {
                timeout: Duration::from_millis(10),
                cooldown: Duration::from_millis(0),
                max_retries: 1,
            },
        )
        .await
        .expect_err("retry should still time out");

        assert!(matches!(err, WslError::WslCommandTimedOut { .. }));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}
