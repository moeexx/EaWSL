pub use crate::services::system::PathVolumeSpace;

#[tauri::command]
pub async fn get_path_volume_space(path: String) -> Result<PathVolumeSpace, String> {
    crate::services::system::get_path_volume_space(path).await
}
