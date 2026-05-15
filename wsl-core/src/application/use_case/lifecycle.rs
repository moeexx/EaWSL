use crate::application::port::wsl_cli::WslLifecyclePort;
use crate::domain::policy::distro::validate_distro_input;
use crate::{WslCommandContext, WslError};

pub(crate) async fn set_default_distro<P>(wsl_cli: &P, distro: &str) -> Result<(), WslError>
where
    P: WslLifecyclePort + ?Sized,
{
    validate_distro_input(WslCommandContext::SetDefault, distro)?;
    wsl_cli.set_default_distro(distro).await
}

pub(crate) async fn terminate_distro<P>(wsl_cli: &P, distro: &str) -> Result<(), WslError>
where
    P: WslLifecyclePort + ?Sized,
{
    validate_distro_input(WslCommandContext::Terminate, distro)?;
    wsl_cli.terminate_distro(distro).await
}

pub(crate) async fn shutdown_wsl<P>(wsl_cli: &P, force: bool) -> Result<(), WslError>
where
    P: WslLifecyclePort + ?Sized,
{
    wsl_cli.shutdown_wsl(force).await
}

pub(crate) async fn unregister_distro<P>(wsl_cli: &P, distro: &str) -> Result<(), WslError>
where
    P: WslLifecyclePort + ?Sized,
{
    validate_distro_input(WslCommandContext::Unregister, distro)?;
    wsl_cli.unregister_distro(distro).await
}

#[cfg(test)]
mod tests {
    use super::{set_default_distro, shutdown_wsl, terminate_distro, unregister_distro};
    use crate::application::port::wsl_cli::WslLifecyclePort;
    use crate::application::use_case::test_support::CallLog;
    use crate::WslError;

    #[derive(Default)]
    struct FakeLifecyclePort {
        calls: CallLog,
    }

    impl FakeLifecyclePort {
        fn calls(&self) -> Vec<String> {
            self.calls.calls()
        }

        fn has_no_calls(&self) -> bool {
            self.calls.is_empty()
        }
    }

    #[allow(async_fn_in_trait)]
    impl WslLifecyclePort for FakeLifecyclePort {
        async fn set_default_distro(&self, distro: &str) -> Result<(), WslError> {
            self.calls.record(format!("set_default:{distro}"));
            Ok(())
        }

        async fn terminate_distro(&self, distro: &str) -> Result<(), WslError> {
            self.calls.record(format!("terminate:{distro}"));
            Ok(())
        }

        async fn shutdown_wsl(&self, force: bool) -> Result<(), WslError> {
            self.calls.record(format!("shutdown:{force}"));
            Ok(())
        }

        async fn unregister_distro(&self, distro: &str) -> Result<(), WslError> {
            self.calls.record(format!("unregister:{distro}"));
            Ok(())
        }
    }

    #[tokio::test]
    async fn lifecycle_use_cases_validate_before_delegating() {
        let wsl_cli = FakeLifecyclePort::default();

        let err = unregister_distro(&wsl_cli, "docker-desktop")
            .await
            .expect_err("restricted distro should fail in application layer");
        assert!(matches!(err, WslError::OperationNotPermitted { .. }));
        assert!(wsl_cli.has_no_calls());

        set_default_distro(&wsl_cli, "Ubuntu")
            .await
            .expect("valid distro should delegate");
        terminate_distro(&wsl_cli, "Ubuntu")
            .await
            .expect("terminate should delegate");
        shutdown_wsl(&wsl_cli, true)
            .await
            .expect("shutdown should delegate");

        assert_eq!(
            wsl_cli.calls(),
            vec![
                "set_default:Ubuntu".to_string(),
                "terminate:Ubuntu".to_string(),
                "shutdown:true".to_string()
            ]
        );
    }
}
