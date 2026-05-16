use std::{future::Future, io, pin::Pin, process::Stdio};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug)]
pub(crate) struct CommandOutput {
    pub status_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub(crate) trait WslCommandRunner {
    fn run<'a, FStarted, FStdout, FStderr>(
        &'a self,
        args: &'a [&'a str],
        cancel_token: Option<tokio_util::sync::CancellationToken>,
        on_started: FStarted,
        on_stdout: FStdout,
        on_stderr: FStderr,
    ) -> Pin<Box<dyn Future<Output = io::Result<CommandOutput>> + Send + 'a>>
    where
        FStarted: FnMut() + Send + 'a,
        FStdout: FnMut(&[u8]) + Send + 'a,
        FStderr: FnMut(&[u8]) + Send + 'a;
}

pub(crate) struct SystemWslRunner;

impl WslCommandRunner for SystemWslRunner {
    fn run<'a, FStarted, FStdout, FStderr>(
        &'a self,
        args: &'a [&'a str],
        cancel_token: Option<tokio_util::sync::CancellationToken>,
        on_started: FStarted,
        on_stdout: FStdout,
        on_stderr: FStderr,
    ) -> Pin<Box<dyn Future<Output = io::Result<CommandOutput>> + Send + 'a>>
    where
        FStarted: FnMut() + Send + 'a,
        FStdout: FnMut(&[u8]) + Send + 'a,
        FStderr: FnMut(&[u8]) + Send + 'a,
    {
        Box::pin(async move {
            run_system_command(args, cancel_token, on_started, on_stdout, on_stderr).await
        })
    }
}

async fn run_system_command<FStarted, FStdout, FStderr>(
    args: &[&str],
    cancel_token: Option<tokio_util::sync::CancellationToken>,
    mut on_started: FStarted,
    on_stdout: FStdout,
    on_stderr: FStderr,
) -> io::Result<CommandOutput>
where
    FStarted: FnMut() + Send,
    FStdout: FnMut(&[u8]) + Send,
    FStderr: FnMut(&[u8]) + Send,
{
    let mut command = Command::new("wsl.exe");
    hide_child_console_window(&mut command);
    command.args(args);
    command.kill_on_drop(true);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("missing stderr pipe"))?;

    on_started();

    let (stdout_bytes, stderr_bytes, status) = match cancel_token {
        Some(token) => {
            let res = tokio::select! {
                res = async {
                    tokio::try_join!(
                        read_stream(stdout, on_stdout),
                        read_stream(stderr, on_stderr),
                        child.wait(),
                    )
                } => Some(res),
                _ = token.cancelled() => None,
            };

            match res {
                Some(r) => r?,
                None => {
                    let _ = cleanup_child(&mut child).await;
                    return Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "operation cancelled",
                    ));
                }
            }
        }
        None => tokio::try_join!(
            read_stream(stdout, on_stdout),
            read_stream(stderr, on_stderr),
            child.wait(),
        )?,
    };

    Ok(CommandOutput {
        status_code: status.code(),
        stdout: stdout_bytes,
        stderr: stderr_bytes,
    })
}

async fn cleanup_child(child: &mut Child) -> io::Result<()> {
    match child.kill().await {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
        Err(error) => return Err(error),
    }

    match child.wait().await {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => Ok(()),
        Err(error) => Err(error),
    }
}

async fn read_stream<R, F>(mut reader: R, mut on_chunk: F) -> io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
    F: FnMut(&[u8]) + Send,
{
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }

        let chunk = &buffer[..read];
        on_chunk(chunk);
        bytes.extend_from_slice(chunk);
    }

    Ok(bytes)
}

#[cfg(windows)]
fn hide_child_console_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_child_console_window(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use std::process::Stdio;

    use super::cleanup_child;

    #[tokio::test]
    async fn cleanup_child_terminates_spawned_process() {
        let mut child = tokio::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 30",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn powershell sleep");

        cleanup_child(&mut child)
            .await
            .expect("cleanup should terminate child");

        let status = child.try_wait().expect("query child status");
        assert!(status.is_some());
    }
}
