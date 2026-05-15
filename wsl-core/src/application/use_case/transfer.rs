use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::application::port::progress_sink::ProgressSink;
use crate::application::port::wsl_cli::WslTransferPort;
use crate::domain::policy::distro::validate_distro_input;
use crate::domain::policy::install::validate_install_options;
use crate::{ExportFormat, InstallOptions, WslCommandContext, WslError};

pub(crate) async fn install_distro<P, S>(
    wsl_cli: &P,
    distro: &str,
    opts: InstallOptions,
    sink: &S,
    cancel_token: CancellationToken,
) -> Result<(), WslError>
where
    P: WslTransferPort + ?Sized,
    S: ProgressSink + ?Sized,
{
    validate_distro_input(WslCommandContext::Install, distro)?;
    validate_install_options(&opts)?;
    wsl_cli
        .install_distro(distro, opts, sink, cancel_token)
        .await
}

pub(crate) async fn import_distro<P, S>(
    wsl_cli: &P,
    distro: &str,
    location: &Path,
    file: &Path,
    sink: &S,
    cancel_token: CancellationToken,
) -> Result<(), WslError>
where
    P: WslTransferPort + ?Sized,
    S: ProgressSink + ?Sized,
{
    validate_distro_input(WslCommandContext::Import, distro)?;
    wsl_cli
        .import_distro(distro, location, file, sink, cancel_token)
        .await
}

pub(crate) async fn import_distro_in_place<P, S>(
    wsl_cli: &P,
    distro: &str,
    vhdx: &Path,
    sink: &S,
    cancel_token: CancellationToken,
) -> Result<(), WslError>
where
    P: WslTransferPort + ?Sized,
    S: ProgressSink + ?Sized,
{
    validate_distro_input(WslCommandContext::ImportInPlace, distro)?;
    wsl_cli
        .import_distro_in_place(distro, vhdx, sink, cancel_token)
        .await
}

pub(crate) async fn export_distro<P, S>(
    wsl_cli: &P,
    distro: &str,
    file: &Path,
    format: ExportFormat,
    sink: &S,
    cancel_token: CancellationToken,
) -> Result<(), WslError>
where
    P: WslTransferPort + ?Sized,
    S: ProgressSink + ?Sized,
{
    validate_distro_input(WslCommandContext::Export, distro)?;
    wsl_cli
        .export_distro(distro, file, format, sink, cancel_token)
        .await
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;

    use tokio_util::sync::CancellationToken;

    use super::{export_distro, import_distro, import_distro_in_place, install_distro};
    use crate::application::port::progress_sink::ProgressSink;
    use crate::application::port::wsl_cli::WslTransferPort;
    use crate::application::use_case::test_support::CallLog;
    use crate::{
        ExportFormat, InstallOptions, ProgressEvent, ProgressPhase, ProgressState, ProgressValue,
        WslError,
    };

    #[derive(Default)]
    struct RecordingProgressSink {
        events: Mutex<Vec<ProgressEvent>>,
    }

    impl RecordingProgressSink {
        fn events(&self) -> Vec<ProgressEvent> {
            self.events.lock().expect("events mutex poisoned").clone()
        }
    }

    impl ProgressSink for RecordingProgressSink {
        async fn emit(&self, event: ProgressEvent) {
            self.events
                .lock()
                .expect("events mutex poisoned")
                .push(event);
        }
    }

    #[derive(Default)]
    struct FakeTransferPort {
        calls: CallLog,
    }

    impl FakeTransferPort {
        fn calls(&self) -> Vec<String> {
            self.calls.calls()
        }

        fn has_no_calls(&self) -> bool {
            self.calls.is_empty()
        }
    }

    #[allow(async_fn_in_trait)]
    impl WslTransferPort for FakeTransferPort {
        async fn install_distro<S>(
            &self,
            distro: &str,
            opts: InstallOptions,
            sink: &S,
            _cancel_token: CancellationToken,
        ) -> Result<(), WslError>
        where
            S: ProgressSink + ?Sized,
        {
            self.calls.record(format!(
                "install:{distro}:{}:{}:{}",
                opts.name.as_deref().unwrap_or("none"),
                opts.fixed_vhd,
                opts.vhd_size
                    .as_ref()
                    .map(|size| size.as_str())
                    .unwrap_or("none")
            ));
            sink.emit(ProgressEvent {
                phase: ProgressPhase::Installing,
                value: ProgressValue::Status(ProgressState::Running),
            })
            .await;
            Ok(())
        }

        async fn import_distro<S>(
            &self,
            distro: &str,
            location: &Path,
            file: &Path,
            _sink: &S,
            _cancel_token: CancellationToken,
        ) -> Result<(), WslError>
        where
            S: ProgressSink + ?Sized,
        {
            self.calls.record(format!(
                "import:{distro}:{}:{}",
                location.display(),
                file.display()
            ));
            Ok(())
        }

        async fn import_distro_in_place<S>(
            &self,
            distro: &str,
            vhdx: &Path,
            _sink: &S,
            _cancel_token: CancellationToken,
        ) -> Result<(), WslError>
        where
            S: ProgressSink + ?Sized,
        {
            self.calls
                .record(format!("import_in_place:{distro}:{}", vhdx.display()));
            Ok(())
        }

        async fn export_distro<S>(
            &self,
            distro: &str,
            file: &Path,
            format: ExportFormat,
            _sink: &S,
            _cancel_token: CancellationToken,
        ) -> Result<(), WslError>
        where
            S: ProgressSink + ?Sized,
        {
            self.calls
                .record(format!("export:{distro}:{}:{format:?}", file.display()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn install_use_case_validates_before_invoking_port() {
        let wsl_cli = FakeTransferPort::default();
        let sink = RecordingProgressSink::default();

        let err = install_distro(
            &wsl_cli,
            "Ubuntu",
            InstallOptions {
                name: None,
                location: None,
                vhd_size: None,
                fixed_vhd: true,
            },
            &sink,
            CancellationToken::new(),
        )
        .await
        .expect_err("invalid install options should fail before delegating");

        assert!(matches!(err, WslError::InvalidArgument { .. }));
        assert!(wsl_cli.has_no_calls());
        assert!(sink.events().is_empty());
    }

    #[tokio::test]
    async fn install_use_case_rejects_invalid_install_name_before_invoking_port() {
        let wsl_cli = FakeTransferPort::default();
        let sink = RecordingProgressSink::default();

        let err = install_distro(
            &wsl_cli,
            "Ubuntu",
            InstallOptions {
                name: Some("My Ubuntu".to_string()),
                location: None,
                vhd_size: None,
                fixed_vhd: false,
            },
            &sink,
            CancellationToken::new(),
        )
        .await
        .expect_err("invalid install name should fail before delegating");

        assert!(matches!(err, WslError::InvalidArgument { .. }));
        assert!(wsl_cli.has_no_calls());
        assert!(sink.events().is_empty());
    }

    #[tokio::test]
    async fn transfer_use_cases_validate_and_delegate() {
        let wsl_cli = FakeTransferPort::default();
        let sink = RecordingProgressSink::default();

        import_distro(
            &wsl_cli,
            "Ubuntu",
            Path::new("D:/WSL/Ubuntu"),
            Path::new("D:/images/ubuntu.tar"),
            &sink,
            CancellationToken::new(),
        )
        .await
        .expect("import should delegate");

        import_distro_in_place(
            &wsl_cli,
            "Ubuntu",
            Path::new("D:/WSL/ubuntu.vhdx"),
            &sink,
            CancellationToken::new(),
        )
        .await
        .expect("import-in-place should delegate");

        export_distro(
            &wsl_cli,
            "docker-desktop",
            Path::new("D:/images/docker-desktop.tar"),
            ExportFormat::Tar,
            &sink,
            CancellationToken::new(),
        )
        .await
        .expect("docker-desktop export remains allowed");

        install_distro(
            &wsl_cli,
            "Ubuntu",
            valid_install_options(),
            &sink,
            CancellationToken::new(),
        )
        .await
        .expect("install should delegate");

        assert_eq!(
            wsl_cli.calls(),
            vec![
                "import:Ubuntu:D:/WSL/Ubuntu:D:/images/ubuntu.tar".to_string(),
                "import_in_place:Ubuntu:D:/WSL/ubuntu.vhdx".to_string(),
                "export:docker-desktop:D:/images/docker-desktop.tar:Tar".to_string(),
                "install:Ubuntu:custom-ubuntu:false:20GB".to_string(),
            ]
        );
        assert_eq!(sink.events().len(), 1);
    }

    fn valid_install_options() -> InstallOptions {
        InstallOptions {
            name: Some("custom-ubuntu".to_string()),
            location: Some("D:/WSL/Ubuntu".into()),
            vhd_size: Some(crate::DiskSize::parse("20GB").expect("valid disk size")),
            fixed_vhd: false,
        }
    }
}
