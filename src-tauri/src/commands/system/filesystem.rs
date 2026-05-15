pub use crate::services::system::FileSystemPathProbe;

#[tauri::command]
pub async fn probe_file_system_path(
    path: String,
    child_limit: Option<u32>,
) -> Result<FileSystemPathProbe, String> {
    crate::services::system::probe_file_system_path(path, child_limit).await
}
