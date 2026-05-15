use std::path::Path;

use crate::application::port::wsl_cli::WslStoragePort;
use crate::domain::policy::distro::validate_distro_input;
use crate::{DiskSize, WslCommandContext, WslError};

pub(crate) async fn move_distro<P>(
    wsl_cli: &P,
    distro: &str,
    new_location: &Path,
) -> Result<(), WslError>
where
    P: WslStoragePort + ?Sized,
{
    validate_distro_input(WslCommandContext::MoveDistro, distro)?;
    wsl_cli.move_distro(distro, new_location).await
}

pub(crate) async fn resize_distro<P>(
    wsl_cli: &P,
    distro: &str,
    size: DiskSize,
) -> Result<(), WslError>
where
    P: WslStoragePort + ?Sized,
{
    validate_distro_input(WslCommandContext::ResizeDistro, distro)?;
    wsl_cli.resize_distro(distro, size).await
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{move_distro, resize_distro};
    use crate::application::port::wsl_cli::WslStoragePort;
    use crate::application::use_case::test_support::CallLog;
    use crate::{DiskSize, WslError};

    #[derive(Default)]
    struct FakeStoragePort {
        calls: CallLog,
    }

    impl FakeStoragePort {
        fn calls(&self) -> Vec<String> {
            self.calls.calls()
        }

        fn has_no_calls(&self) -> bool {
            self.calls.is_empty()
        }
    }

    #[allow(async_fn_in_trait)]
    impl WslStoragePort for FakeStoragePort {
        async fn move_distro(&self, distro: &str, new_location: &Path) -> Result<(), WslError> {
            self.calls
                .record(format!("move:{distro}:{}", new_location.display()));
            Ok(())
        }

        async fn resize_distro(&self, distro: &str, size: DiskSize) -> Result<(), WslError> {
            self.calls
                .record(format!("resize:{distro}:{}", size.as_str()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn storage_use_cases_validate_and_delegate() {
        let wsl_cli = FakeStoragePort::default();

        let err = move_distro(
            &wsl_cli,
            "docker-desktop",
            Path::new("D:/WSL/docker-desktop"),
        )
        .await
        .expect_err("protected distro move should fail");
        assert!(matches!(err, WslError::OperationNotPermitted { .. }));
        assert!(wsl_cli.has_no_calls());

        move_distro(&wsl_cli, "Ubuntu", Path::new("D:/WSL/NewUbuntu"))
            .await
            .expect("valid move should delegate");
        resize_distro(
            &wsl_cli,
            "Ubuntu",
            DiskSize::parse("20GB").expect("valid disk size"),
        )
        .await
        .expect("valid resize should delegate");
        assert_eq!(
            wsl_cli.calls(),
            vec![
                "move:Ubuntu:D:/WSL/NewUbuntu".to_string(),
                "resize:Ubuntu:20GB".to_string(),
            ]
        );
    }
}
