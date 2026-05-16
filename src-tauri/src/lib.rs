mod bridge;
pub mod commands;
mod services;

use tauri::Manager;

fn register_handlers(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        commands::system::file_size::get_file_size,
        commands::system::filesystem::probe_file_system_path,
        commands::system::volume_space::get_path_volume_space,
        commands::system::overview::get_system_overview,
        commands::settings::get_app_settings,
        commands::settings::save_app_settings,
        commands::long_tasks::get_long_tasks,
        commands::long_tasks::save_long_tasks,
        commands::wsl::query::get_wsl_version,
        commands::wsl::query::list_distros,
        commands::wsl::query::list_online_distros,
        commands::wsl::lifecycle::set_default_distro,
        commands::wsl::lifecycle::terminate_distro,
        commands::wsl::lifecycle::shutdown_wsl,
        commands::wsl::lifecycle::unregister_distro,
        commands::wsl::transfer::install_distro,
        commands::wsl::transfer::import_distro,
        commands::wsl::transfer::import_distro_in_place,
        commands::wsl::transfer::export_distro,
    ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    register_handlers(tauri::Builder::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
