use tauri::AppHandle;

pub use crate::services::distro_metadata::DistroMetadata;

#[tauri::command]
pub async fn get_distro_metadata(app: AppHandle) -> Result<Vec<DistroMetadata>, String> {
    crate::services::distro_metadata::get(app).await
}

#[tauri::command]
pub async fn refresh_distro_metadata(app: AppHandle) -> Result<Vec<DistroMetadata>, String> {
    crate::services::distro_metadata::refresh(app).await
}
