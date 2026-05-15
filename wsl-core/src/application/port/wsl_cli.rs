use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::application::port::progress_sink::ProgressSink;
use crate::domain::model::distro::InstalledDistroSnapshot;
use crate::{DiskSize, ExportFormat, InstallOptions, OnlineDistro, WslError, WslVersion};

#[allow(async_fn_in_trait)]
pub(crate) trait WslQueryPort: Send + Sync {
    async fn get_wsl_version(&self) -> Result<WslVersion, WslError>;
    async fn list_installed_distros(&self) -> Result<Vec<InstalledDistroSnapshot>, WslError>;
    async fn list_online_distros(&self) -> Result<Vec<OnlineDistro>, WslError>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait WslLifecyclePort: Send + Sync {
    async fn set_default_distro(&self, distro: &str) -> Result<(), WslError>;
    async fn terminate_distro(&self, distro: &str) -> Result<(), WslError>;
    async fn shutdown_wsl(&self, force: bool) -> Result<(), WslError>;
    async fn unregister_distro(&self, distro: &str) -> Result<(), WslError>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait WslStoragePort: Send + Sync {
    async fn move_distro(&self, distro: &str, new_location: &Path) -> Result<(), WslError>;
    async fn resize_distro(&self, distro: &str, size: DiskSize) -> Result<(), WslError>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait WslTransferPort: Send + Sync {
    async fn install_distro<S>(
        &self,
        distro: &str,
        opts: InstallOptions,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized;
    async fn import_distro<S>(
        &self,
        distro: &str,
        location: &Path,
        file: &Path,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized;
    async fn import_distro_in_place<S>(
        &self,
        distro: &str,
        vhdx: &Path,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized;
    async fn export_distro<S>(
        &self,
        distro: &str,
        file: &Path,
        format: ExportFormat,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized;
}
