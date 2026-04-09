use tauri::{AppHandle, Emitter};

use switchyard_core::{self, PanelState};

fn codex_home() -> Result<std::path::PathBuf, String> {
    switchyard_core::resolve_codex_home().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_panel_state() -> Result<PanelState, String> {
    let codex_home = codex_home()?;
    switchyard_core::load_panel_state(&codex_home).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn switch_account(app: AppHandle, account_key: String) -> Result<PanelState, String> {
    let codex_home = codex_home()?;
    let state = switchyard_core::switch_account(&codex_home, &account_key)
        .map_err(|error| error.to_string())?;
    let _ = app.emit("switchyard://registry-changed", ());
    Ok(state)
}

#[tauri::command]
pub fn set_auto_switch(app: AppHandle, enabled: bool) -> Result<PanelState, String> {
    let codex_home = codex_home()?;
    let state = switchyard_core::set_auto_switch_enabled(&codex_home, enabled)
        .map_err(|error| error.to_string())?;
    let _ = app.emit("switchyard://registry-changed", ());
    Ok(state)
}

#[tauri::command]
pub fn set_usage_api(app: AppHandle, enabled: bool) -> Result<PanelState, String> {
    let codex_home = codex_home()?;
    let state = switchyard_core::set_usage_api_enabled(&codex_home, enabled)
        .map_err(|error| error.to_string())?;
    let _ = app.emit("switchyard://registry-changed", ());
    Ok(state)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
