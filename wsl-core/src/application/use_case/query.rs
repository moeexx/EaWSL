use crate::application::port::distro_registry::DistroRegistryPort;
use crate::application::port::wsl_cli::WslQueryPort;
use crate::domain::policy::aggregation::aggregate_distros;
use crate::{DistroInfo, OnlineDistro, WslError, WslVersion};

pub(crate) async fn get_wsl_version<P>(wsl_cli: &P) -> Result<WslVersion, WslError>
where
    P: WslQueryPort + ?Sized,
{
    wsl_cli.get_wsl_version().await
}

pub(crate) async fn list_distros<P, R>(
    wsl_cli: &P,
    registry: &R,
) -> Result<Vec<DistroInfo>, WslError>
where
    P: WslQueryPort + ?Sized,
    R: DistroRegistryPort + ?Sized,
{
    let cli_entries = wsl_cli.list_installed_distros().await?;
    let registry_entries = registry.read_all_distros()?;
    Ok(aggregate_distros(cli_entries, registry_entries))
}

pub(crate) async fn list_online_distros<P>(wsl_cli: &P) -> Result<Vec<OnlineDistro>, WslError>
where
    P: WslQueryPort + ?Sized,
{
    wsl_cli.list_online_distros().await
}

#[cfg(test)]
mod tests {
    use super::{get_wsl_version, list_distros, list_online_distros};
    use crate::application::port::distro_registry::DistroRegistryPort;
    use crate::application::port::wsl_cli::WslQueryPort;
    use crate::application::use_case::test_support::CallLog;
    use crate::domain::model::distro::{InstalledDistroSnapshot, RegisteredDistroMetadata};
    use crate::{DistroInfo, DistroState, OnlineDistro, WslError, WslVersion};

    struct FakeQueryPort {
        calls: CallLog,
        version: WslVersion,
        installed: Vec<InstalledDistroSnapshot>,
        online: Vec<OnlineDistro>,
    }

    impl FakeQueryPort {
        fn new() -> Self {
            Self {
                calls: CallLog::default(),
                version: wsl_version(),
                installed: vec![installed_distro()],
                online: vec![OnlineDistro {
                    name: "Ubuntu".to_string(),
                    friendly_name: "Ubuntu".to_string(),
                }],
            }
        }

        fn with_installed(installed: Vec<InstalledDistroSnapshot>) -> Self {
            Self {
                installed,
                ..Self::new()
            }
        }

        fn calls(&self) -> Vec<String> {
            self.calls.calls()
        }
    }

    #[allow(async_fn_in_trait)]
    impl WslQueryPort for FakeQueryPort {
        async fn get_wsl_version(&self) -> Result<WslVersion, WslError> {
            self.calls.record("get_wsl_version");
            Ok(self.version.clone())
        }

        async fn list_installed_distros(&self) -> Result<Vec<InstalledDistroSnapshot>, WslError> {
            self.calls.record("list_installed_distros");
            Ok(self.installed.clone())
        }

        async fn list_online_distros(&self) -> Result<Vec<OnlineDistro>, WslError> {
            self.calls.record("list_online_distros");
            Ok(self.online.clone())
        }
    }

    struct FakeRegistry {
        entries: Vec<RegisteredDistroMetadata>,
        should_fail: bool,
    }

    impl FakeRegistry {
        fn ok(entries: Vec<RegisteredDistroMetadata>) -> Self {
            Self {
                entries,
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                entries: Vec::new(),
                should_fail: true,
            }
        }
    }

    impl DistroRegistryPort for FakeRegistry {
        fn read_all_distros(&self) -> Result<Vec<RegisteredDistroMetadata>, WslError> {
            if self.should_fail {
                return Err(WslError::RegistryReadFailed {
                    key: Some("HKCU\\...\\Lxss".to_string()),
                    detail: "boom".to_string(),
                });
            }

            Ok(self.entries.clone())
        }
    }

    #[tokio::test]
    async fn query_use_cases_delegate_and_aggregate() {
        let wsl_cli = FakeQueryPort::new();
        let registry = FakeRegistry::ok(vec![registry_metadata()]);

        let version = get_wsl_version(&wsl_cli)
            .await
            .expect("version should delegate");
        assert_eq!(version.wsl, "2.6.3.0");

        let online = list_online_distros(&wsl_cli)
            .await
            .expect("online query should delegate");
        assert_eq!(online[0].name, "Ubuntu");

        let distros = list_distros(&wsl_cli, &registry)
            .await
            .expect("list_distros should aggregate");
        assert_eq!(
            distros,
            vec![DistroInfo {
                name: "Ubuntu".to_string(),
                state: crate::DistroState::Running,
                version: 2,
                is_default: true,
                base_path: Some("D:/WSL/Ubuntu".into()),
                vhd_file_name: Some("ext4.vhdx".to_string()),
                flavor: Some("ubuntu".to_string()),
                os_version: Some("24.04".to_string()),
                default_uid: Some(1000),
            }]
        );
        assert_eq!(
            wsl_cli.calls(),
            vec![
                "get_wsl_version".to_string(),
                "list_online_distros".to_string(),
                "list_installed_distros".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn list_distros_propagates_registry_errors() {
        let wsl_cli = FakeQueryPort::with_installed(vec![installed_distro()]);
        let registry = FakeRegistry::failing();

        let err = list_distros(&wsl_cli, &registry)
            .await
            .expect_err("registry errors should bubble up");
        assert!(matches!(err, WslError::RegistryReadFailed { .. }));
    }

    fn wsl_version() -> WslVersion {
        WslVersion {
            wsl: "2.6.3.0".to_string(),
            kernel: None,
            wslg: None,
            msrdc: None,
            direct3d: None,
            dxcore: None,
            windows: "10.0.26200.8117".to_string(),
        }
    }

    fn installed_distro() -> InstalledDistroSnapshot {
        InstalledDistroSnapshot {
            name: "Ubuntu".to_string(),
            state: DistroState::Running,
            version: 2,
            is_default: true,
        }
    }

    fn registry_metadata() -> RegisteredDistroMetadata {
        RegisteredDistroMetadata {
            name: "Ubuntu".to_string(),
            version: 2,
            base_path: Some("D:/WSL/Ubuntu".into()),
            vhd_file_name: Some("ext4.vhdx".to_string()),
            flavor: Some("ubuntu".to_string()),
            os_version: Some("24.04".to_string()),
            default_uid: Some(1000),
        }
    }
}
