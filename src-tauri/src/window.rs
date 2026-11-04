use tauri::{AppHandle, Manager, Window, WindowEvent};

pub const MAIN: &str = "main";

pub fn show_settings(handle: &AppHandle) {
    if let Some(window) = handle.get_webview_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == MAIN {
            api.prevent_close();
            let _ = window.hide();
            if let Some(state) = window.app_handle().try_state::<crate::state::AppState>() {
                state.audio.stop_preview();
            }
        }
    }
}
