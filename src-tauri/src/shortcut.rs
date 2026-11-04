use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{Builder, GlobalShortcutExt, Shortcut, ShortcutState};

use crate::actions;
use crate::error::{AppError, AppResult};

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                actions::toggle_sound(app);
            }
        })
        .build()
}

pub fn apply(handle: &AppHandle, accelerator: Option<&str>) -> AppResult<()> {
    let manager = handle.global_shortcut();
    manager.unregister_all().map_err(|e| AppError::Other(e.to_string()))?;

    let Some(accelerator) = accelerator.filter(|a| !a.trim().is_empty()) else {
        return Ok(());
    };
    let shortcut: Shortcut = accelerator
        .parse()
        .map_err(|e: <Shortcut as std::str::FromStr>::Err| AppError::InvalidShortcut(e.to_string()))?;
    manager.register(shortcut).map_err(|e| AppError::InvalidShortcut(e.to_string()))
}
