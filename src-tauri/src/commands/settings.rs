use tauri::AppHandle;

pub use crate::services::settings_service::{
    AppSettings, BackgroundRefreshSettings, BackgroundRefreshTarget,
};

#[tauri::command]
pub async fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    crate::services::settings_service::get(app).await
}

#[tauri::command]
pub async fn save_app_settings(
    app: AppHandle,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    crate::services::settings_service::save(app, settings).await
}
