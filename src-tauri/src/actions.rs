use std::thread;
use std::time::{Duration, Instant};

use app_watch::{AppEvent, AppIdentity, ProcessScanner};
use tauri::{AppHandle, Manager};

use crate::config::Config;
use crate::state::AppState;

const WINDOW_POLL: Duration = Duration::from_millis(250);
const WINDOW_TIMEOUT: Duration = Duration::from_secs(60);

pub fn toggle_sound(handle: &AppHandle) {
    let state = handle.state::<AppState>();
    let result = state.update(handle, |c| {
        c.settings.sound_enabled = !c.settings.sound_enabled;
        Ok(())
    });
    if let Err(e) = result {
        log::error!("소리 켜기/끄기를 저장하지 못했습니다: {e}");
    }
}

pub fn handle_app_event(handle: &AppHandle, event: AppEvent) {
    let state = handle.state::<AppState>();
    let config = state.snapshot();
    let requests = state.engine().plan(&config, &event, Instant::now());

    for request in requests {
        let path = state.store.sound_path(&request.file);
        let audio = state.audio.clone();
        let delay = Duration::from_millis(request.delay_ms as u64);

        if !request.wait_for_window && delay.is_zero() {
            audio.play(path, request.volume);
            continue;
        }

        let app = matching_app(&config, &event);
        thread::spawn(move || {
            if request.wait_for_window {
                if let Some(app) = app {
                    if !wait_for_main_window(&app) {
                        return; 
                    }
                }
            }
            thread::sleep(delay);
            audio.play(path, request.volume);
        });
    }
}

fn matching_app(config: &Config, event: &AppEvent) -> Option<AppIdentity> {
    config.rules.iter().find(|r| r.app.matches(&event.process)).map(|r| r.app.clone())
}

fn wait_for_main_window(app: &AppIdentity) -> bool {
    let mut scanner = ProcessScanner::new();
    let started = Instant::now();
    while started.elapsed() < WINDOW_TIMEOUT {
        let pids: Vec<u32> =
            scanner.scan().into_iter().filter(|p| app.matches(p)).map(|p| p.pid).collect();
        if pids.is_empty() {
            return false;
        }
        match app_watch::window::has_main_window(&pids) {
            Some(true) | None => return true,
            Some(false) => thread::sleep(WINDOW_POLL),
        }
    }
    false
}