pub(crate) mod settings;
pub(crate) mod shared;
pub(crate) mod system;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod wsl;

pub use crate::bridge::progress::{
    DistroProgressEvent, TransferProgressEvent, TransferProgressPhase, TransferProgressValue,
};
pub use settings::{
    get_app_settings, save_app_settings, AppSettings, BackgroundRefreshSettings,
    BackgroundRefreshTarget,
};
pub use shared::dto::{
    ExportDistroRequest, ImportDistroInPlaceRequest, ImportDistroRequest, InstallDistroRequest,
    InstallOptionsPayload,
};
pub use system::{
    get_file_size, get_path_volume_space, get_system_overview, probe_file_system_path,
    FileSystemPathProbe, PathVolumeSpace, SystemOverview, SystemOverviewScope,
};
pub use wsl::lifecycle::{set_default_distro, shutdown_wsl, terminate_distro, unregister_distro};
pub use wsl::query::{get_wsl_version, list_distros, list_online_distros};
pub use wsl::transfer::{export_distro, import_distro, import_distro_in_place, install_distro};
