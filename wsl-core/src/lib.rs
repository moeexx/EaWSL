//! EaWSL core crate with the stable public API.

mod application;
mod domain;
mod infrastructure;

use std::path::Path;

use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use application::port::progress_sink::ChannelProgressSink;
use infrastructure::registry::adapter::SystemDistroRegistryAdapter;
use infrastructure::wsl_cli::adapter::SystemWslCliAdapter;

pub use domain::error::{WslCommandContext, WslError};
pub use domain::model::distro::{DistroInfo, DistroState, OnlineDistro, WslVersion};
pub use domain::model::progress::{ProgressEvent, ProgressPhase, ProgressState, ProgressValue};
pub use domain::value::install::InstallOptions;
pub use domain::value::storage::{DiskSize, DiskSizeParseError, ExportFormat};

pub async fn get_wsl_version() -> Result<WslVersion, WslError> {
    application::use_case::query::get_wsl_version(&SystemWslCliAdapter).await
}

pub async fn list_distros() -> Result<Vec<DistroInfo>, WslError> {
    application::use_case::query::list_distros(&SystemWslCliAdapter, &SystemDistroRegistryAdapter)
        .await
}

pub async fn list_online_distros() -> Result<Vec<OnlineDistro>, WslError> {
    application::use_case::query::list_online_distros(&SystemWslCliAdapter).await
}

pub async fn set_default_distro(distro: &str) -> Result<(), WslError> {
    application::use_case::lifecycle::set_default_distro(&SystemWslCliAdapter, distro).await
}

pub async fn terminate_distro(distro: &str) -> Result<(), WslError> {
    application::use_case::lifecycle::terminate_distro(&SystemWslCliAdapter, distro).await
}

pub async fn shutdown_wsl(force: bool) -> Result<(), WslError> {
    application::use_case::lifecycle::shutdown_wsl(&SystemWslCliAdapter, force).await
}

pub async fn unregister_distro(distro: &str) -> Result<(), WslError> {
    application::use_case::lifecycle::unregister_distro(&SystemWslCliAdapter, distro).await
}

pub async fn move_distro(distro: &str, new_location: &Path) -> Result<(), WslError> {
    application::use_case::storage::move_distro(&SystemWslCliAdapter, distro, new_location).await
}

pub async fn resize_distro(distro: &str, size: DiskSize) -> Result<(), WslError> {
    application::use_case::storage::resize_distro(&SystemWslCliAdapter, distro, size).await
}

pub async fn install_distro(
    distro: &str,
    opts: InstallOptions,
    tx: Sender<ProgressEvent>,
    cancel_token: CancellationToken,
) -> Result<(), WslError> {
    let sink = ChannelProgressSink::new(tx);
    application::use_case::transfer::install_distro(
        &SystemWslCliAdapter,
        distro,
        opts,
        &sink,
        cancel_token,
    )
    .await
}

pub async fn import_distro(
    distro: &str,
    location: &Path,
    file: &Path,
    tx: Sender<ProgressEvent>,
    cancel_token: CancellationToken,
) -> Result<(), WslError> {
    let sink = ChannelProgressSink::new(tx);
    application::use_case::transfer::import_distro(
        &SystemWslCliAdapter,
        distro,
        location,
        file,
        &sink,
        cancel_token,
    )
    .await
}

pub async fn import_distro_in_place(
    distro: &str,
    vhdx: &Path,
    tx: Sender<ProgressEvent>,
    cancel_token: CancellationToken,
) -> Result<(), WslError> {
    let sink = ChannelProgressSink::new(tx);
    application::use_case::transfer::import_distro_in_place(
        &SystemWslCliAdapter,
        distro,
        vhdx,
        &sink,
        cancel_token,
    )
    .await
}

pub async fn export_distro(
    distro: &str,
    file: &Path,
    format: ExportFormat,
    tx: Sender<ProgressEvent>,
    cancel_token: CancellationToken,
) -> Result<(), WslError> {
    let sink = ChannelProgressSink::new(tx);
    application::use_case::transfer::export_distro(
        &SystemWslCliAdapter,
        distro,
        file,
        format,
        &sink,
        cancel_token,
    )
    .await
}
