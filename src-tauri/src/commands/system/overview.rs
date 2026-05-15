pub use crate::services::system::{SystemOverview, SystemOverviewScope};

#[tauri::command]
pub async fn get_system_overview(
    scope: Option<SystemOverviewScope>,
) -> Result<SystemOverview, String> {
    crate::services::system::get_system_overview(scope).await
}
