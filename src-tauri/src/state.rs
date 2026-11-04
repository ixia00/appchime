use std::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, Emitter};

use crate::audio::AudioPlayer;
use crate::config::{Config, Store};
use crate::engine::Engine;
use crate::error::AppResult;
use crate::{autostart, tray};

pub const EVENT_CONFIG_CHANGED: &str = "config-changed";

pub struct AppState {
    config: Mutex<Config>,
    pub store: Store,
    pub audio: AudioPlayer,
    pub engine: Mutex<Engine>,
}

impl AppState {
    pub fn new(store: Store, config: Config, audio: AudioPlayer) -> Self {
        Self { config: Mutex::new(config), store, audio, engine: Mutex::new(Engine::default()) }
    }

    pub fn snapshot(&self) -> Config {
        lock(&self.config).clone()
    }

    pub fn update<R>(
        &self,
        handle: &AppHandle,
        f: impl FnOnce(&mut Config) -> AppResult<R>,
    ) -> AppResult<R> {
        let (old, new, result) = {
            let mut guard = lock(&self.config);
            let old = guard.clone();
            let mut next = old.clone();
            let result = f(&mut next)?;
            if next == old {
                return Ok(result);
            }
            self.store.save(&next)?;
            *guard = next.clone();
            (old, next, result)
        };
        on_changed(handle, &old, &new);
        Ok(result)
    }

    pub fn engine(&self) -> MutexGuard<'_, Engine> {
        lock(&self.engine)
    }
}

fn on_changed(handle: &AppHandle, old: &Config, new: &Config) {
    if let Err(e) = handle.emit(EVENT_CONFIG_CHANGED, new) {
        log::warn!("화면에 변경 알림을 보내지 못했습니다: {e}");
    }
    tray::refresh(handle, new);
    if old.settings.launch_at_login != new.settings.launch_at_login {
        autostart::apply(handle, new.settings.launch_at_login);
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
