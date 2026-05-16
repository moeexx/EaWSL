use eawsl_tauri::commands::{
    self, AppSettings, BackgroundRefreshSettings, BackgroundRefreshTarget, DistroProgressEvent,
    ExportDistroRequest, FileSystemPathProbe, ImportDistroInPlaceRequest, ImportDistroRequest,
    InstallDistroRequest, InstallOptionsPayload, PathVolumeSpace, PersistedLongTask,
    SystemOverview, SystemOverviewScope, TransferProgressEvent, TransferProgressPhase,
    TransferProgressValue,
};
use wsl_core::{ExportFormat, ProgressState};

#[test]
fn command_public_surface_type_checks() {
    let _ = InstallDistroRequest {
        request_id: "req-install".to_string(),
        distro: "Ubuntu".to_string(),
        options: InstallOptionsPayload {
            name: None,
            location: Some("D:/WSL/Ubuntu".into()),
            vhd_size: Some("20GB".to_string()),
            fixed_vhd: false,
        },
    };

    let _ = DistroProgressEvent {
        request_id: "req-progress".to_string(),
        distro: "Ubuntu".to_string(),
        progress: TransferProgressEvent {
            phase: TransferProgressPhase::Installing,
            value: TransferProgressValue::Status(ProgressState::Running),
        },
    };

    let _ = ImportDistroRequest {
        request_id: "req-import".to_string(),
        distro: "Ubuntu".to_string(),
        location: "D:/WSL/Ubuntu".into(),
        file: "D:/images/ubuntu.tar".into(),
    };

    let _ = ImportDistroInPlaceRequest {
        request_id: "req-vhdx".to_string(),
        distro: "Ubuntu".to_string(),
        source_vhdx: "D:/images/ext4.vhdx".into(),
        target_directory: Some("D:/WSL/Ubuntu".into()),
    };

    let _ = ExportDistroRequest {
        request_id: "req-export".to_string(),
        distro: "Ubuntu".to_string(),
        file: "D:/exports/ubuntu.tar.gz".into(),
        format: ExportFormat::TarGz,
    };

    let _: Option<SystemOverview> = None;
    let _: Option<SystemOverviewScope> = None;
    let _ = PathVolumeSpace {
        volume_root: "C:\\".to_string(),
        free_bytes: 1,
    };
    let _ = FileSystemPathProbe {
        exists: true,
        is_file: false,
        is_dir: true,
        direct_child_count: Some(1),
        child_count_limit_exceeded: false,
        direct_vhdx_file_count: Some(1),
        has_direct_children: Some(true),
    };
    let _ = AppSettings {
        default_install_location: "D:/WSL".to_string(),
        background_refresh: BackgroundRefreshSettings {
            interval_minutes: 15,
            targets: vec![
                BackgroundRefreshTarget::Distros,
                BackgroundRefreshTarget::SystemOverviewStorage,
                BackgroundRefreshTarget::WslVersion,
            ],
        },
    };
    let _ = PersistedLongTask {
        request_id: "req-task".to_string(),
        distro: "Ubuntu".to_string(),
        operation: "export".to_string(),
        status: "completed".to_string(),
        phase: Some("Exporting".to_string()),
        percent: Some(100.0),
        started_at: "2026-05-16T00:00:00.000Z".to_string(),
        ended_at: Some("2026-05-16T00:01:00.000Z".to_string()),
        error: None,
        location: Some("D:/exports/ubuntu.tar".to_string()),
        interrupted: false,
    };

    let _ = commands::get_file_size;
    let _ = commands::probe_file_system_path;
    let _ = commands::get_path_volume_space;
    let _ = commands::get_system_overview;
    let _ = commands::get_app_settings;
    let _ = commands::save_app_settings;
    let _ = commands::get_long_tasks;
    let _ = commands::save_long_tasks;
    let _ = commands::get_wsl_version;
    let _ = commands::list_distros;
    let _ = commands::list_online_distros;
    let _ = commands::set_default_distro;
    let _ = commands::terminate_distro;
    let _ = commands::shutdown_wsl;
    let _ = commands::unregister_distro;
    let _ = commands::install_distro;
    let _ = commands::import_distro;
    let _ = commands::import_distro_in_place;
    let _ = commands::export_distro;
}
