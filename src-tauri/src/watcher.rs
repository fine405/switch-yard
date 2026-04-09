use std::sync::Mutex;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{App, Emitter, Manager, Runtime};

pub struct RegistryWatcher {
    _watcher: Mutex<Option<RecommendedWatcher>>,
}

pub fn install<R: Runtime>(app: &mut App<R>) {
    let watcher = create_watcher(app.handle());
    app.manage(RegistryWatcher {
        _watcher: Mutex::new(watcher.ok()),
    });
}

fn create_watcher<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> notify::Result<RecommendedWatcher> {
    let codex_home = match switchyard_core::resolve_codex_home() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Switchyard: 无法启动文件监听，{}", error);
            return Err(notify::Error::generic("missing-codex-home"));
        }
    };

    if !codex_home.exists() {
        return Err(notify::Error::generic("missing-codex-root"));
    }

    let handle = app_handle.clone();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if event.paths.iter().any(|path| is_relevant_path(path)) {
                let _ = handle.emit("switchyard://registry-changed", ());
            }
        }
    })?;

    watcher.watch(&codex_home, RecursiveMode::Recursive)?;
    Ok(watcher)
}

fn is_relevant_path(path: &std::path::Path) -> bool {
    matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some("registry.json") | Some("auth.json")
    )
}
