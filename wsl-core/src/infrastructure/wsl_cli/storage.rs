use std::path::Path;

use crate::infrastructure::wsl_cli::command::{
    run_no_output_command_with_policy, WSL_ACTION_POLICY,
};
use crate::infrastructure::wsl_cli::runner::{SystemWslRunner, WslCommandRunner};
use crate::{DiskSize, WslCommandContext, WslError};

pub(crate) async fn move_distro(distro: &str, new_location: &Path) -> Result<(), WslError> {
    move_distro_with_runner(&SystemWslRunner, distro, new_location).await
}

pub(crate) async fn resize_distro(distro: &str, size: DiskSize) -> Result<(), WslError> {
    resize_distro_with_runner(&SystemWslRunner, distro, size).await
}

pub(crate) async fn move_distro_with_runner<R>(
    runner: &R,
    distro: &str,
    new_location: &Path,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    let location_arg = new_location.to_string_lossy().into_owned();
    run_no_output_command_with_policy(
        runner,
        WslCommandContext::MoveDistro,
        &["--manage", distro, "--move", location_arg.as_str()],
        WSL_ACTION_POLICY,
    )
    .await
}

pub(crate) async fn resize_distro_with_runner<R>(
    runner: &R,
    distro: &str,
    size: DiskSize,
) -> Result<(), WslError>
where
    R: WslCommandRunner + Sync,
{
    run_no_output_command_with_policy(
        runner,
        WslCommandContext::ResizeDistro,
        &["--manage", distro, "--resize", size.as_str()],
        WSL_ACTION_POLICY,
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{move_distro_with_runner, resize_distro_with_runner};
    use crate::infrastructure::wsl_cli::test_support::FakeRunner;
    use crate::DiskSize;

    #[tokio::test]
    async fn move_distro_passes_expected_args() {
        let runner =
            FakeRunner::ok_empty_with_args(&["--manage", "Ubuntu", "--move", r"D:\WSL\Target"]);
        move_distro_with_runner(&runner, "Ubuntu", Path::new(r"D:\WSL\Target"))
            .await
            .expect("move should succeed");
    }

    #[tokio::test]
    async fn resize_distro_passes_expected_args() {
        let runner = FakeRunner::ok_empty_with_args(&["--manage", "Ubuntu", "--resize", "20GB"]);
        resize_distro_with_runner(
            &runner,
            "Ubuntu",
            DiskSize::parse("20GB").expect("valid disk size"),
        )
        .await
        .expect("resize should succeed");
    }
}
