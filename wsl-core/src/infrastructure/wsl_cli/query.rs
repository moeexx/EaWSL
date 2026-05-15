use crate::domain::model::distro::InstalledDistroSnapshot;
use crate::infrastructure::wsl_cli::command::{run_capture_with_policy, WSL_QUERY_POLICY};
use crate::infrastructure::wsl_cli::parser::list_online::parse_list_online_output;
use crate::infrastructure::wsl_cli::parser::list_verbose::parse_list_verbose_output;
use crate::infrastructure::wsl_cli::parser::version::parse_version_output;
use crate::infrastructure::wsl_cli::runner::{SystemWslRunner, WslCommandRunner};
use crate::{OnlineDistro, WslCommandContext, WslError, WslVersion};

/// Get structured output from `wsl.exe --version`.
pub(crate) async fn get_wsl_version() -> Result<WslVersion, WslError> {
    get_wsl_version_with_runner(&SystemWslRunner).await
}

/// Get command-side snapshots from `wsl.exe --list --verbose`.
pub(crate) async fn list_installed_distros() -> Result<Vec<InstalledDistroSnapshot>, WslError> {
    list_installed_distros_with_runner(&SystemWslRunner).await
}

/// Get installable entries from `wsl.exe --list --online`.
pub(crate) async fn list_online_distros() -> Result<Vec<OnlineDistro>, WslError> {
    list_online_distros_with_runner(&SystemWslRunner).await
}

pub(crate) async fn get_wsl_version_with_runner<R>(runner: &R) -> Result<WslVersion, WslError>
where
    R: WslCommandRunner + Sync,
{
    let output = run_capture_with_policy(
        runner,
        WslCommandContext::Version,
        &["--version"],
        WSL_QUERY_POLICY,
    )
    .await?;
    parse_version_output(&output.stdout)
}

pub(crate) async fn list_installed_distros_with_runner<R>(
    runner: &R,
) -> Result<Vec<InstalledDistroSnapshot>, WslError>
where
    R: WslCommandRunner + Sync,
{
    let output = run_capture_with_policy(
        runner,
        WslCommandContext::ListVerbose,
        &["--list", "--verbose"],
        WSL_QUERY_POLICY,
    )
    .await?;
    parse_list_verbose_output(&output.stdout)
}

pub(crate) async fn list_online_distros_with_runner<R>(
    runner: &R,
) -> Result<Vec<OnlineDistro>, WslError>
where
    R: WslCommandRunner + Sync,
{
    let output = run_capture_with_policy(
        runner,
        WslCommandContext::ListOnline,
        &["--list", "--online"],
        WSL_QUERY_POLICY,
    )
    .await?;
    parse_list_online_output(&output.stdout)
}

#[cfg(test)]
mod tests {
    use super::{
        get_wsl_version_with_runner, list_installed_distros_with_runner,
        list_online_distros_with_runner,
    };
    use crate::domain::model::distro::InstalledDistroSnapshot;
    use crate::infrastructure::wsl_cli::runner::CommandOutput;
    use crate::infrastructure::wsl_cli::test_support::{encode_utf16le, FakeRunner};
    use crate::{DistroState, OnlineDistro, WslCommandContext, WslError};

    #[tokio::test]
    async fn version_adapter_passes_expected_args_and_parses_output() {
        let runner = FakeRunner::ok_utf16_with_args(
            &["--version"],
            "WSL version: 2.6.3.0\r\nWindows version: 10.0.26200.8039\r\n",
        );

        let version = get_wsl_version_with_runner(&runner)
            .await
            .expect("parse version");
        assert_eq!(version.wsl, "2.6.3.0");
        assert_eq!(version.windows, "10.0.26200.8039");
    }

    #[tokio::test]
    async fn version_fails_on_missing_required_field() {
        let runner = FakeRunner::ok_utf16("WSL version: 2.6.3.0\r\n");

        let err = get_wsl_version_with_runner(&runner)
            .await
            .expect_err("missing windows field");
        match err {
            WslError::OutputParseFailed {
                context,
                detail,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::Version);
                assert!(detail.contains("Windows version field"));
                assert!(raw_output.contains("WSL version"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn version_fails_on_duplicate_known_label() {
        let runner = FakeRunner::ok_utf16(
            "WSL version: 2.6.3.0\r\nWSL 版本: 2.6.3.1\r\nWindows: 10.0.26200.8039\r\n",
        );

        let err = get_wsl_version_with_runner(&runner)
            .await
            .expect_err("duplicate labels should fail");
        assert!(matches!(err, WslError::OutputParseFailed { .. }));
    }

    #[tokio::test]
    async fn list_verbose_parses_default_unknown_state_and_special_characters() {
        let runner = FakeRunner::ok_utf16_with_args(
            &["--list", "--verbose"],
            "NAME              STATE           VERSION\r\n* Debian          Stopped         2\r\nedge-test_01     Paused          1\r\n",
        );

        let entries = list_installed_distros_with_runner(&runner)
            .await
            .expect("parse list --verbose");
        assert_eq!(
            entries,
            vec![
                InstalledDistroSnapshot {
                    name: "Debian".to_string(),
                    state: DistroState::Stopped,
                    version: 2,
                    is_default: true,
                },
                InstalledDistroSnapshot {
                    name: "edge-test_01".to_string(),
                    state: DistroState::Unknown("Paused".to_string()),
                    version: 1,
                    is_default: false,
                },
            ]
        );
    }

    #[tokio::test]
    async fn list_verbose_returns_empty_for_header_only() {
        let runner = FakeRunner::ok_utf16("NAME              STATE           VERSION\r\n");

        let entries = list_installed_distros_with_runner(&runner)
            .await
            .expect("header only should return empty list");
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn list_verbose_rejects_names_with_spaces() {
        let runner = FakeRunner::ok_utf16(
            "NAME              STATE           VERSION\r\nMy Ubuntu         Stopped         2\r\n",
        );

        let err = list_installed_distros_with_runner(&runner)
            .await
            .expect_err("names with spaces should fail");
        assert!(matches!(err, WslError::OutputParseFailed { .. }));
    }

    #[tokio::test]
    async fn list_verbose_rejects_invalid_versions() {
        let runner = FakeRunner::ok_utf16(
            "NAME              STATE           VERSION\r\nUbuntu            Stopped         two\r\n",
        );

        let err = list_installed_distros_with_runner(&runner)
            .await
            .expect_err("invalid version should fail");
        assert!(matches!(err, WslError::OutputParseFailed { .. }));
    }

    #[tokio::test]
    async fn list_online_parses_preamble_and_friendly_names() {
        let runner = FakeRunner::ok_utf16_with_args(
            &["--list", "--online"],
            "以下是可安装的有效分发的列表。使用“wsl.exe --install <Distro>”安装。\r\n\r\nNAME                            FRIENDLY NAME\r\nUbuntu                          Ubuntu\r\nUbuntu-24.04                    Ubuntu 24.04 LTS\r\nDebian                          Debian GNU/Linux\r\n",
        );

        let entries = list_online_distros_with_runner(&runner)
            .await
            .expect("parse list --online");
        assert_eq!(
            entries,
            vec![
                OnlineDistro {
                    name: "Ubuntu".to_string(),
                    friendly_name: "Ubuntu".to_string(),
                },
                OnlineDistro {
                    name: "Ubuntu-24.04".to_string(),
                    friendly_name: "Ubuntu 24.04 LTS".to_string(),
                },
                OnlineDistro {
                    name: "Debian".to_string(),
                    friendly_name: "Debian GNU/Linux".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn list_online_returns_empty_for_header_only() {
        let runner = FakeRunner::ok_utf16("NAME                            FRIENDLY NAME\r\n");

        let entries = list_online_distros_with_runner(&runner)
            .await
            .expect("header only should return empty list");
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn list_online_rejects_invalid_rows() {
        let runner = FakeRunner::ok_utf16(
            "NAME                            FRIENDLY NAME\r\nUbuntu 24.04 LTS\r\n",
        );

        let err = list_online_distros_with_runner(&runner)
            .await
            .expect_err("invalid row should fail");
        assert!(matches!(err, WslError::OutputParseFailed { .. }));
    }

    #[tokio::test]
    async fn version_extracts_invalid_argument_from_stdout() {
        let runner = FakeRunner::from_result(Ok(CommandOutput {
            status_code: Some(-1),
            stdout: encode_utf16le(
                "请使用“wsl.exe --help”获取受支持的参数列表。错误代码: Wsl/E_INVALIDARG\r\n",
            ),
            stderr: Vec::new(),
        }));

        let err = get_wsl_version_with_runner(&runner)
            .await
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
}
