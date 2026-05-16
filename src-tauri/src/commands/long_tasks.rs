use tauri::AppHandle;

pub use crate::services::long_task_history::PersistedLongTask;

#[tauri::command]
pub async fn get_long_tasks(app: AppHandle) -> Result<Vec<PersistedLongTask>, String> {
    crate::services::long_task_history::get(app).await
}

#[tauri::command]
pub async fn save_long_tasks(app: AppHandle, tasks: Vec<PersistedLongTask>) -> Result<(), String> {
    crate::services::long_task_history::save(app, tasks).await
}
