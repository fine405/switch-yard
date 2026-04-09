mod commands;
mod panel;
mod watcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .enable_macos_default_menu(false)
        .setup(|app| {
            panel::setup(app)?;
            watcher::install(app);
            Ok(())
        })
        .on_window_event(|window, event| panel::handle_window_event(window, event))
        .invoke_handler(tauri::generate_handler![
            commands::load_panel_state,
            commands::switch_account,
            commands::set_auto_switch,
            commands::set_usage_api,
            commands::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
