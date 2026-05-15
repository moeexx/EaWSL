use crate::commands::shared::error::map_command_error;

#[tauri::command]
pub async fn set_default_distro(distro: String) -> Result<(), String> {
    wsl_core::set_default_distro(&distro)
        .await
        .map_err(map_command_error)
}

#[tauri::command]
pub async fn terminate_distro(distro: String) -> Result<(), String> {
    wsl_core::terminate_distro(&distro)
        .await
        .map_err(map_command_error)
}

#[tauri::command]
pub async fn shutdown_wsl(force: bool) -> Result<(), String> {
    wsl_core::shutdown_wsl(force)
        .await
        .map_err(map_command_error)
}

#[tauri::command]
pub async fn unregister_distro(distro: String) -> Result<(), String> {
    wsl_core::unregister_distro(&distro)
        .await
        .map_err(map_command_error)
}
