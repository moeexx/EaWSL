#[tauri::command]
pub async fn get_file_size(path: String) -> Result<u64, String> {
    crate::services::system::get_file_size(path).await
}
