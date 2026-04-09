use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, Runtime, Window, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

pub const PANEL_LABEL: &str = "main";

const MENU_TOGGLE_PANEL: &str = "toggle-panel";
const MENU_QUIT: &str = "quit";

pub fn setup<R: Runtime>(app: &mut App<R>) -> tauri::Result<()> {
    let toggle_item =
        tauri::menu::MenuItem::with_id(app, MENU_TOGGLE_PANEL, "显示面板", true, None::<&str>)?;
    let quit_item =
        tauri::menu::MenuItem::with_id(app, MENU_QUIT, "退出 Switchyard", true, None::<&str>)?;
    let tray_menu = MenuBuilder::new(app)
        .item(&toggle_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("missing default window icon");

    TrayIconBuilder::with_id("switchyard")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("Switchyard")
        .show_menu_on_left_click(false)
        .menu(&tray_menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_TOGGLE_PANEL => {
                let _ = toggle_panel(app);
            }
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = toggle_panel(tray.app_handle());
            }
        })
        .build(app)?;

    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        #[cfg(target_os = "macos")]
        let _ = window.set_visible_on_all_workspaces(true);
        let _ = window.hide();
    }

    Ok(())
}

pub fn handle_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != PANEL_LABEL {
        return;
    }

    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        WindowEvent::Focused(focused) => {
            if !focused {
                let _ = window.hide();
            }
        }
        _ => {}
    }
}

pub fn toggle_panel<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<()> {
    let Some(window) = manager.get_webview_window(PANEL_LABEL) else {
        return Ok(());
    };

    if window.is_visible()? {
        window.hide()?;
        return Ok(());
    }

    window.move_window_constrained(Position::TrayCenter)?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}
