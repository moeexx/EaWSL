use crate::infrastructure::wsl_cli::command::{
    run_no_output_command_with_policy, WSL_ACTION_POLICY,
};
use crate::infrastructure::wsl_cli::runner::{SystemWslRunner, WslCommandRunner};
use crate::{WslCommandContext, WslError};

pub(crate) async fn set_default_distro(distro: &str) -> Result<(), WslError> {
    set_default_distro_with_runner(&SystemWslRunner, distro).await
}

pub(crate) async fn terminate_distro(distro: &str) -> Result<(), WslError> {
    terminate_distro_with_runner(&SystemWslRunner, distro).await
}

pub(crate) async fn shutdown_wsl(force: bool) -> Result<(), WslError> {
    shutdown_wsl_with_runner(&SystemWslRunner, force).await
}

pub(crate) async fn unregister_distro(distro: &str) -> Result<(), WslError> {
    unregister_distro_with_runner(&SystemWslRunner, distro).await
}

pub(crate) async fn set_default_distro_with_runner<R>(
    runner: &R,
    distro: &str,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    run_no_output_command_with_policy(
        runner,
        WslCommandContext::SetDefault,
        &["--set-default", distro],
        WSL_ACTION_POLICY,
    )
    .await
}

pub(crate) async fn terminate_distro_with_runner<R>(
    runner: &R,
    distro: &str,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    run_no_output_command_with_policy(
        runner,
        WslCommandContext::Terminate,
        &["--terminate", distro],
        WSL_ACTION_POLICY,
    )
    .await
}

pub(crate) async fn shutdown_wsl_with_runner<R>(runner: &R, force: bool) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    let args = if force {
        vec!["--shutdown", "--force"]
    } else {
        vec!["--shutdown"]
    };

    run_no_output_command_with_policy(
        runner,
        WslCommandContext::Shutdown,
        &args,
        WSL_ACTION_POLICY,
    )
    .await
}

pub(crate) async fn unregister_distro_with_runner<R>(
    runner: &R,
    distro: &str,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    run_no_output_command_with_policy(
        runner,
        WslCommandContext::Unregister,
        &["--unregister", distro],
        WSL_ACTION_POLICY,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{
        set_default_distro_with_runner, shutdown_wsl_with_runner, terminate_distro_with_runner,
        unregister_distro_with_runner,
    };
    use crate::infrastructure::wsl_cli::test_support::FakeRunner;

    #[tokio::test]
    async fn set_default_passes_expected_args() {
        let runner = FakeRunner::ok_empty_with_args(&["--set-default", "Ubuntu"]);
        set_default_distro_with_runner(&runner, "Ubuntu")
            .await
            .expect("set default should succeed");
    }

    #[tokio::test]
    async fn terminate_passes_expected_args() {
        let runner = FakeRunner::ok_empty_with_args(&["--terminate", "Ubuntu"]);
        terminate_distro_with_runner(&runner, "Ubuntu")
            .await
            .expect("terminate should succeed");
    }

    #[tokio::test]
    async fn shutdown_builds_expected_args() {
        let runner = FakeRunner::ok_empty_with_args(&["--shutdown"]);
        shutdown_wsl_with_runner(&runner, false)
            .await
            .expect("shutdown should succeed");

        let runner = FakeRunner::ok_empty_with_args(&["--shutdown", "--force"]);
        shutdown_wsl_with_runner(&runner, true)
            .await
            .expect("forced shutdown should succeed");
    }

    #[tokio::test]
    async fn unregister_passes_expected_args() {
        let runner = FakeRunner::ok_empty_with_args(&["--unregister", "Ubuntu"]);
        unregister_distro_with_runner(&runner, "Ubuntu")
            .await
            .expect("unregister should succeed");
    }
}
