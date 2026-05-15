use std::path::Path;

use tokio::sync::mpsc;
use wsl_core::{
    DiskSize, DistroInfo, DistroState, ExportFormat, InstallOptions, OnlineDistro, ProgressEvent,
    ProgressPhase, ProgressState, ProgressValue, WslCommandContext, WslError, WslVersion,
};

#[test]
fn public_types_are_constructible() {
    let _ = DistroInfo {
        name: "Ubuntu".to_string(),
        state: DistroState::Running,
        version: 2,
        is_default: false,
        base_path: Some("D:/WSL/Ubuntu".into()),
        vhd_file_name: Some("ext4.vhdx".to_string()),
        flavor: Some("ubuntu".to_string()),
        os_version: Some("24.04".to_string()),
        default_uid: Some(1000),
    };

    let _ = WslVersion {
        wsl: "2.6.3.0".to_string(),
        kernel: Some("6.6.87.2-1".to_string()),
        wslg: Some("1.0.71".to_string()),
        msrdc: Some("1.2.6353".to_string()),
        direct3d: Some("1.611.1-81528511".to_string()),
        dxcore: Some("10.0.26100.1-240331-1435.ge-release".to_string()),
        windows: "10.0.26200.8117".to_string(),
    };

    let _ = OnlineDistro {
        name: "Ubuntu-24.04".to_string(),
        friendly_name: "Ubuntu 24.04 LTS".to_string(),
    };

    let _ = InstallOptions {
        name: Some("custom-ubuntu".to_string()),
        location: Some("D:/WSL/Ubuntu".into()),
        vhd_size: Some(DiskSize::parse("20GB").expect("valid disk size")),
        fixed_vhd: true,
    };

    let _ = ProgressEvent {
        phase: ProgressPhase::Installing,
        value: ProgressValue::Status(ProgressState::Running),
    };

    let _ = ExportFormat::TarGz;
    let _ = WslCommandContext::Export;
    let _ = WslError::RegistryReadFailed {
        key: Some("HKEY_CURRENT_USER\\...\\Lxss".to_string()),
        detail: "not implemented".to_string(),
    };
}

#[test]
fn public_function_signatures_type_check() {
    std::mem::drop(wsl_core::get_wsl_version());
    std::mem::drop(wsl_core::list_distros());
    std::mem::drop(wsl_core::list_online_distros());
    std::mem::drop(wsl_core::set_default_distro("Ubuntu"));
    std::mem::drop(wsl_core::terminate_distro("Ubuntu"));
    std::mem::drop(wsl_core::shutdown_wsl(true));
    std::mem::drop(wsl_core::unregister_distro("Ubuntu"));
    std::mem::drop(wsl_core::move_distro(
        "Ubuntu",
        Path::new("D:/WSL/NewUbuntu"),
    ));
    std::mem::drop(wsl_core::resize_distro(
        "Ubuntu",
        DiskSize::parse("20GB").expect("valid disk size"),
    ));
    let (tx, _rx) = mpsc::channel::<ProgressEvent>(8);
    std::mem::drop(wsl_core::install_distro(
        "Ubuntu",
        InstallOptions {
            name: None,
            location: Some("D:/WSL/Ubuntu".into()),
            vhd_size: Some(DiskSize::parse("20GB").expect("valid disk size")),
            fixed_vhd: false,
        },
        tx,
        tokio_util::sync::CancellationToken::new(),
    ));

    let (tx, _rx) = mpsc::channel::<ProgressEvent>(8);
    std::mem::drop(wsl_core::import_distro(
        "Ubuntu",
        Path::new("D:/WSL/Ubuntu"),
        Path::new("D:/images/ubuntu.tar"),
        tx,
        tokio_util::sync::CancellationToken::new(),
    ));

    let (tx, _rx) = mpsc::channel::<ProgressEvent>(8);
    std::mem::drop(wsl_core::import_distro_in_place(
        "Ubuntu",
        Path::new("D:/WSL/ubuntu.vhdx"),
        tx,
        tokio_util::sync::CancellationToken::new(),
    ));

    let (tx, _rx) = mpsc::channel::<ProgressEvent>(8);
    std::mem::drop(wsl_core::export_distro(
        "Ubuntu",
        Path::new("D:/images/ubuntu.tar.gz"),
        ExportFormat::TarGz,
        tx,
        tokio_util::sync::CancellationToken::new(),
    ));
}
