mod actions;
mod audio;
mod autostart;
mod commands;
mod config;
mod engine;
mod error;
mod i18n;
mod shortcut;
mod sounds;
mod state;
mod tray;
mod window;
mod install;

use std::time::Duration;

use app_watch::Watcher;
use tauri::{App, Manager};
use tauri_plugin_autostart::MacosLauncher;

use crate::audio::AudioPlayer;
use crate::config::Store;
use crate::state::AppState;

const WATCH_INTERVAL: Duration = Duration::from_millis(1000);

pub fn run() {
    if install::relocate_if_needed() {
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| window::show_settings(app)))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![autostart::LAUNCH_ARG]),
        ))
        .plugin(shortcut::plugin())
        .setup(|app| {
            setup(app)?;
            Ok(())
        })
        .on_window_event(window::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::get_meta,
            commands::get_config,
            commands::list_running_apps,
            commands::list_installed_apps,
            commands::identity_from_path,
            commands::update_settings,
            commands::add_rule,
            commands::update_rule,
            commands::remove_rule,
            commands::add_sounds,
            commands::remove_sound,
            commands::reorder_sound,
            commands::preview_sound,
            commands::stop_preview,
            commands::accessibility_status,
            commands::quit_app,
            commands::app_icon,
        ])
        .run(tauri::generate_context!())
        .expect("AppChime을 시작하지 못했습니다");
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let handle = app.handle().clone();
    let paths = handle.path();
    let store = Store::new(&paths.app_config_dir()?, &paths.app_data_dir()?)?;
    let loaded = store.load();
    let config = loaded.config;

    app.manage(AppState::new(store, config.clone(), AudioPlayer::spawn()?));

    tray::create(&handle, &config)?;

    if let Err(e) = shortcut::apply(&handle, config.settings.toggle_shortcut.as_deref()) {
        log::warn!("저장된 단축키를 등록하지 못했습니다: {e}");
    }
    if loaded.first_run {
        let _ = handle.state::<AppState>().store.save(&config);
        autostart::apply(&handle, config.settings.launch_at_login);
    } else {
        autostart::sync(&handle, config.settings.launch_at_login);
    }

    let event_handle = handle.clone();
    let watcher = Watcher::spawn(WATCH_INTERVAL, move |event| {
        actions::handle_app_event(&event_handle, event);
    })?;
    app.manage(watcher);

    if (loaded.first_run || config.rules.is_empty()) && !autostart::launched_by_autostart() {
        window::show_settings(&handle);
    }
    Ok(())
}
