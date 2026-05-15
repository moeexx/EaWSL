use std::{path::Path, sync::Mutex};

use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::infrastructure::wsl_cli::command::{
    export_format_arg, interpret_command_result, run_command, spawn_progress_forwarder,
};
use crate::infrastructure::wsl_cli::parser::progress::{
    CommandProgressHandler, ExportProgressHandler, ImportProgressHandler, InstallProgressHandler,
};
use crate::infrastructure::wsl_cli::runner::{SystemWslRunner, WslCommandRunner};
use crate::{ExportFormat, InstallOptions, ProgressEvent, WslCommandContext, WslError};

pub(crate) async fn install_distro(
    distro: &str,
    opts: InstallOptions,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError> {
    install_distro_with_runner(&SystemWslRunner, distro, opts, tx, cancel_token).await
}

pub(crate) async fn import_distro(
    distro: &str,
    location: &Path,
    file: &Path,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError> {
    import_distro_with_runner(&SystemWslRunner, distro, location, file, tx, cancel_token).await
}

pub(crate) async fn import_distro_in_place(
    distro: &str,
    vhdx: &Path,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError> {
    import_distro_in_place_with_runner(&SystemWslRunner, distro, vhdx, tx, cancel_token).await
}

pub(crate) async fn export_distro(
    distro: &str,
    file: &Path,
    format: ExportFormat,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError> {
    export_distro_with_runner(&SystemWslRunner, distro, file, format, tx, cancel_token).await
}

async fn run_progress_command<R, H, F>(
    runner: &R,
    args: Vec<String>,
    context: WslCommandContext,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
    build_handler: F,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
    H: CommandProgressHandler,
    F: FnOnce(UnboundedSender<ProgressEvent>) -> H,
{
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (event_tx, forwarder) = spawn_progress_forwarder(tx);
    let handler = Mutex::new(build_handler(event_tx.clone()));

    let mut on_started = || {
        handler.lock().expect("progress handler lock").on_started();
    };
    let mut on_stdout = |chunk: &[u8]| {
        handler
            .lock()
            .expect("progress handler lock")
            .on_stdout_chunk(chunk);
    };
    let mut on_stderr = |chunk: &[u8]| {
        handler
            .lock()
            .expect("progress handler lock")
            .on_stderr_chunk(chunk);
    };

    let run_result = run_command(
        runner,
        &arg_refs,
        Some(cancel_token),
        &mut on_started,
        &mut on_stdout,
        &mut on_stderr,
    )
    .await;
    handler.lock().expect("progress handler lock").finish();

    let result = match run_result {
        Ok(decoded) => match interpret_command_result(context, decoded) {
            Ok(_) => {
                handler.lock().expect("progress handler lock").on_success();
                Ok(())
            }
            Err(err) => Err(err),
        },
        Err(err) => Err(err),
    };

    drop(handler);
    drop(event_tx);
    let _ = forwarder.await;

    result
}

pub(crate) async fn import_distro_in_place_with_runner<R>(
    runner: &R,
    distro: &str,
    vhdx: &Path,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    let args = build_import_in_place_args(distro, vhdx);
    run_progress_command(
        runner,
        args,
        WslCommandContext::ImportInPlace,
        tx,
        cancel_token,
        ImportProgressHandler::new,
    )
    .await
}

pub(crate) async fn install_distro_with_runner<R>(
    runner: &R,
    distro: &str,
    opts: InstallOptions,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    let args = build_install_args(distro, &opts);

    run_progress_command(
        runner,
        args,
        WslCommandContext::Install,
        tx,
        cancel_token,
        InstallProgressHandler::new,
    )
    .await
}

pub(crate) async fn import_distro_with_runner<R>(
    runner: &R,
    distro: &str,
    location: &Path,
    file: &Path,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    let args = build_import_args(distro, location, file);
    run_progress_command(
        runner,
        args,
        WslCommandContext::Import,
        tx,
        cancel_token,
        ImportProgressHandler::new,
    )
    .await
}

pub(crate) async fn export_distro_with_runner<R>(
    runner: &R,
    distro: &str,
    file: &Path,
    format: ExportFormat,
    tx: Sender<ProgressEvent>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    let args = build_export_args(distro, file, &format);
    run_progress_command(
        runner,
        args,
        WslCommandContext::Export,
        tx,
        cancel_token,
        ExportProgressHandler::new,
    )
    .await
}

fn build_import_in_place_args(distro: &str, vhdx: &Path) -> Vec<String> {
    vec![
        "--import-in-place".to_string(),
        distro.to_string(),
        vhdx.to_string_lossy().into_owned(),
    ]
}

fn build_install_args(distro: &str, opts: &InstallOptions) -> Vec<String> {
    let mut args = vec![
        "--install".to_string(),
        distro.to_string(),
        "--no-launch".to_string(),
        "--version".to_string(),
        "2".to_string(),
    ];

    if let Some(name) = opts.name.as_ref() {
        args.push("--name".to_string());
        args.push(name.clone());
    }

    if let Some(location) = opts.location.as_ref() {
        args.push("--location".to_string());
        args.push(location.to_string_lossy().into_owned());
    }

    if let Some(vhd_size) = opts.vhd_size.as_ref() {
        args.push("--vhd-size".to_string());
        args.push(vhd_size.as_str().to_string());
    }

    if opts.fixed_vhd {
        args.push("--fixed-vhd".to_string());
    }

    args
}

fn build_import_args(distro: &str, location: &Path, file: &Path) -> Vec<String> {
    vec![
        "--import".to_string(),
        distro.to_string(),
        location.to_string_lossy().into_owned(),
        file.to_string_lossy().into_owned(),
        "--version".to_string(),
        "2".to_string(),
    ]
}

fn build_export_args(distro: &str, file: &Path, format: &ExportFormat) -> Vec<String> {
    vec![
        "--export".to_string(),
        distro.to_string(),
        file.to_string_lossy().into_owned(),
        "--format".to_string(),
        export_format_arg(format).to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tokio::sync::mpsc;

    use super::{
        export_distro_with_runner, import_distro_in_place_with_runner, import_distro_with_runner,
        install_distro_with_runner,
    };
    use crate::infrastructure::wsl_cli::test_support::{collect_events, FakeRunner};
    use crate::{
        DiskSize, ExportFormat, InstallOptions, ProgressEvent, ProgressPhase, ProgressState,
        ProgressValue, WslCommandContext, WslError,
    };

    #[tokio::test]
    async fn import_in_place_passes_expected_args_and_emits_status_events() {
        let runner = FakeRunner::streaming_utf16(
            &["--import-in-place", "Ubuntu", r"D:\WSL\ubuntu.vhdx"],
            &["操作成功完成。\n"],
            0,
        );
        let (tx, rx) = mpsc::channel(16);

        import_distro_in_place_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\WSL\ubuntu.vhdx"),
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("import-in-place should succeed");

        let events = collect_events(rx).await;
        assert_eq!(
            events,
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[tokio::test]
    async fn install_passes_expected_args_and_emits_progress() {
        let runner = FakeRunner::streaming_utf16(
            &[
                "--install",
                "Ubuntu",
                "--no-launch",
                "--version",
                "2",
                "--name",
                "custom-ubuntu",
                "--location",
                r"D:\WSL\Ubuntu",
                "--vhd-size",
                "20GB",
                "--fixed-vhd",
            ],
            &[
                "[=====20.",
                "4%]\r",
                "[================95.0%]\r",
                "正在安装: Ubuntu\r\n[==5.0%]\r",
            ],
            0,
        );
        let opts = InstallOptions {
            name: Some("custom-ubuntu".to_string()),
            location: Some(r"D:\WSL\Ubuntu".into()),
            vhd_size: Some(DiskSize::parse("20GB").expect("valid disk size")),
            fixed_vhd: true,
        };
        let (tx, rx) = mpsc::channel(16);

        install_distro_with_runner(
            &runner,
            "Ubuntu",
            opts,
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("install should succeed");

        let events = collect_events(rx).await;
        assert_eq!(
            events,
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Downloading,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Downloading,
                    value: ProgressValue::Percent(20.4),
                },
                ProgressEvent {
                    phase: ProgressPhase::Downloading,
                    value: ProgressValue::Percent(95.0),
                },
                ProgressEvent {
                    phase: ProgressPhase::Installing,
                    value: ProgressValue::Percent(5.0),
                },
                ProgressEvent {
                    phase: ProgressPhase::Installing,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[tokio::test]
    async fn import_passes_expected_args_and_emits_progress() {
        let runner = FakeRunner::streaming_utf16(
            &[
                "--import",
                "Ubuntu",
                r"D:\WSL\Ubuntu",
                r"D:\images\ubuntu.tar",
                "--version",
                "2",
            ],
            &["[===========20.", "4%]\n", "操作成功完成。\n"],
            0,
        );
        let (tx, rx) = mpsc::channel(16);

        import_distro_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\WSL\Ubuntu"),
            Path::new(r"D:\images\ubuntu.tar"),
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("import should succeed");

        let events = collect_events(rx).await;
        assert_eq!(
            events,
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Percent(20.4),
                },
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[tokio::test]
    async fn export_passes_expected_args_and_emits_status_events() {
        let runner = FakeRunner::streaming_with_stderr(
            &[
                "--export",
                "Ubuntu",
                r"D:\images\ubuntu.tar.gz",
                "--format",
                "tar.gz",
            ],
            vec![crate::infrastructure::wsl_cli::test_support::encode_utf16le("正在导出\r\n")],
            Vec::new(),
            0,
        );
        let (tx, rx) = mpsc::channel(16);

        export_distro_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\images\ubuntu.tar.gz"),
            ExportFormat::TarGz,
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("export should succeed");

        let events = collect_events(rx).await;
        assert_eq!(
            events,
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Exporting,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Exporting,
                    value: ProgressValue::Status(ProgressState::Running),
                },
                ProgressEvent {
                    phase: ProgressPhase::Exporting,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[tokio::test]
    async fn vhd_export_uses_vhdx_file_and_vhd_format_arg() {
        let runner = FakeRunner::streaming_with_stderr(
            &[
                "--export",
                "Ubuntu",
                r"D:\images\ubuntu.vhdx",
                "--format",
                "vhd",
            ],
            vec![crate::infrastructure::wsl_cli::test_support::encode_utf16le("正在导出\r\n")],
            Vec::new(),
            0,
        );
        let (tx, _rx) = mpsc::channel(16);

        export_distro_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\images\ubuntu.vhdx"),
            ExportFormat::Vhd,
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("vhd export should use .vhdx path with --format vhd");
    }

    #[tokio::test]
    async fn install_extracts_stdout_only_errors() {
        let runner = FakeRunner::streaming_utf16(
            &["--install", "Ubuntu", "--no-launch", "--version", "2"],
            &["请使用“wsl.exe --help”获取受支持的参数列表。错误代码: Wsl/E_INVALIDARG\r\n"],
            -1,
        );
        let opts = InstallOptions::default();
        let (tx, _rx) = mpsc::channel(4);

        let err = install_distro_with_runner(
            &runner,
            "Ubuntu",
            opts,
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect_err("invalid argument should be extracted from stdout");

        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::Install);
                assert!(raw_output.contains("Wsl/E_INVALIDARG"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn import_extracts_stdout_only_file_not_found_errors() {
        let runner = FakeRunner::streaming_utf16(
            &[
                "--import",
                "Ubuntu",
                r"D:\WSL\Ubuntu",
                r"D:\images\missing.tar",
                "--version",
                "2",
            ],
            &["系统找不到指定的文件。\r\n错误代码: Wsl/ERROR_FILE_NOT_FOUND\r\n"],
            -1,
        );
        let (tx, _rx) = mpsc::channel(4);

        let err = import_distro_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\WSL\Ubuntu"),
            Path::new(r"D:\images\missing.tar"),
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect_err("file-not-found should be extracted from stdout");
        assert!(matches!(err, WslError::FileNotFound));
    }

    #[tokio::test]
    async fn export_extracts_stdout_only_distro_not_found_errors() {
        let runner = FakeRunner::streaming_utf16(
            &[
                "--export",
                "Ubuntu",
                r"D:\images\ubuntu.tar",
                "--format",
                "tar",
            ],
            &["不存在具有所提供名称的分发。\r\n错误代码: Wsl/Service/WSL_E_DISTRO_NOT_FOUND\r\n"],
            -1,
        );
        let (tx, _rx) = mpsc::channel(4);

        let err = export_distro_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\images\ubuntu.tar"),
            ExportFormat::Tar,
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect_err("distro-not-found should be extracted from stdout");
        assert!(matches!(err, WslError::DistroNotFound));
    }

    #[tokio::test]
    async fn progress_channel_closure_does_not_fail_import() {
        let runner = FakeRunner::streaming_utf16(
            &[
                "--import",
                "Ubuntu",
                r"D:\WSL\Ubuntu",
                r"D:\images\ubuntu.tar",
                "--version",
                "2",
            ],
            &["[===========20.4%]\n", "操作成功完成。\n"],
            0,
        );
        let (tx, rx) = mpsc::channel(1);
        drop(rx);

        import_distro_with_runner(
            &runner,
            "Ubuntu",
            Path::new(r"D:\WSL\Ubuntu"),
            Path::new(r"D:\images\ubuntu.tar"),
            tx,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("closed progress receiver should not fail import");
    }
}
