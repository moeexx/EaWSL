use wsl_core::{DistroInfo, OnlineDistro, WslVersion};

use crate::commands::shared::error::map_command_error;

#[tauri::command]
pub async fn get_wsl_version() -> Result<WslVersion, String> {
    wsl_core::get_wsl_version().await.map_err(map_command_error)
}

#[tauri::command]
pub async fn list_distros() -> Result<Vec<DistroInfo>, String> {
    wsl_core::list_distros().await.map_err(map_command_error)
}

#[tauri::command]
pub async fn list_online_distros() -> Result<Vec<OnlineDistro>, String> {
    wsl_core::list_online_distros()
        .await
        .map_err(map_command_error)
}
