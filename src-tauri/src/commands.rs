use std::path::{Path, PathBuf};

use app_watch::{catalog, AppIdentity};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::config::{Config, LaunchTiming, PlayMode, Rule, Settings, Sound, TriggerKind};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::{shortcut, sounds};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    platform: &'static str,
    version: String,
    supported_formats: Vec<&'static str>,
}

#[tauri::command]
pub fn get_meta(handle: AppHandle) -> Meta {
    Meta {
        platform: std::env::consts::OS,
        version: handle.package_info().version.to_string(),
        supported_formats: sounds::SUPPORTED_EXTENSIONS.to_vec(),
    }
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Config {
    state.snapshot()
}

#[tauri::command(async)]
pub fn list_running_apps() -> Vec<AppIdentity> {
    catalog::running_apps()
}

#[tauri::command(async)]
pub fn list_installed_apps() -> Vec<AppIdentity> {
    catalog::installed_apps()
}

#[tauri::command(async)]
pub fn identity_from_path(path: PathBuf) -> AppResult<AppIdentity> {
    catalog::identity_from_path(&path).ok_or_else(|| AppError::InvalidApp(path.display().to_string()))
}

#[tauri::command]
pub fn update_settings(handle: AppHandle, state: State<AppState>, settings: Settings) -> AppResult<Config> {
    if settings.dedupe_seconds > 600 {
        return Err(AppError::InvalidInput("dedupeSeconds".into()));
    }

    let previous = state.snapshot().settings.toggle_shortcut;
    if previous != settings.toggle_shortcut {
        if let Err(e) = shortcut::apply(&handle, settings.toggle_shortcut.as_deref()) {
            let _ = shortcut::apply(&handle, previous.as_deref());
            return Err(e);
        }
    }

    state.update(&handle, |c| {
        c.settings = settings;
        Ok(c.clone())
    })
}

#[tauri::command]
pub fn add_rule(handle: AppHandle, state: State<AppState>, app: AppIdentity) -> AppResult<Rule> {
    state.update(&handle, |c| {
        if c.rules.iter().any(|r| r.app.key() == app.key()) {
            return Err(AppError::DuplicateApp);
        }
        let rule = Rule::new(app);
        c.rules.push(rule.clone());
        Ok(rule)
    })
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TriggerPatch {
    enabled: Option<bool>,
    mode: Option<PlayMode>,
    timing: Option<LaunchTiming>,
    delay_ms: Option<u32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RulePatch {
    enabled: Option<bool>,
    volume: Option<f32>,
    on_launch: Option<TriggerPatch>,
    on_quit: Option<TriggerPatch>,
}

#[tauri::command]
pub fn update_rule(handle: AppHandle, state: State<AppState>, id: String, patch: RulePatch) -> AppResult<Rule> {
    let rule = state.update(&handle, |c| {
        let rule = c.rule_mut(&id).ok_or(AppError::NotFound)?;
        if let Some(v) = patch.enabled {
            rule.enabled = v;
        }
        if let Some(v) = patch.volume {
            if !(0.0..=1.0).contains(&v) {
                return Err(AppError::InvalidInput("volume".into()));
            }
            rule.volume = v;
        }
        for (kind, p) in [(TriggerKind::Launch, patch.on_launch), (TriggerKind::Quit, patch.on_quit)] {
            let Some(p) = p else { continue };
            let trigger = rule.trigger_mut(kind);
            if let Some(v) = p.enabled {
                trigger.enabled = v;
            }
            if let Some(v) = p.mode {
                trigger.mode = v;
            }
            if let Some(v) = p.timing {
                trigger.timing = v;
            }
            if let Some(v) = p.delay_ms {
                trigger.delay_ms = v.min(30_000);
            }
        }
        Ok(rule.clone())
    })?;
    state.engine().forget_rule(&id);
    Ok(rule)
}

#[tauri::command]
pub fn remove_rule(handle: AppHandle, state: State<AppState>, id: String) -> AppResult<()> {
    let removed = state.update(&handle, |c| {
        let index = c.rules.iter().position(|r| r.id == id).ok_or(AppError::NotFound)?;
        Ok(c.rules.remove(index))
    })?;
    for sound in removed.all_sounds() {
        sounds::remove(state.store.sounds_dir(), sound);
    }
    state.engine().forget_rule(&id);
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedFile {
    name: String,
    error: AppError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSoundsResult {
    rule: Rule,
    failed: Vec<FailedFile>,
}

#[tauri::command(async)]
pub fn add_sounds(
    handle: AppHandle,
    state: State<'_, AppState>,
    rule_id: String,
    kind: TriggerKind,
    paths: Vec<PathBuf>,
) -> AppResult<AddSoundsResult> {
    let dir = state.store.sounds_dir();
    let mut imported: Vec<Sound> = Vec::new();
    let mut failed = Vec::new();
    for path in &paths {
        match sounds::import(path, dir) {
            Ok(sound) => imported.push(sound),
            Err(error) => failed.push(FailedFile { name: file_name(path), error }),
        }
    }

    let to_add = imported.clone();
    let result = state.update(&handle, move |c| {
        let rule = c.rule_mut(&rule_id).ok_or(AppError::NotFound)?;
        rule.trigger_mut(kind).sounds.extend(to_add);
        Ok(rule.clone())
    });

    match result {
        Ok(rule) => {
            state.engine().forget_rule(&rule.id);
            Ok(AddSoundsResult { rule, failed })
        }
        Err(e) => {
            for sound in &imported {
                sounds::remove(dir, sound);
            }
            Err(e)
        }
    }
}

#[tauri::command]
pub fn remove_sound(
    handle: AppHandle,
    state: State<AppState>,
    rule_id: String,
    kind: TriggerKind,
    sound_id: String,
) -> AppResult<Rule> {
    let (rule, removed) = state.update(&handle, |c| {
        let rule = c.rule_mut(&rule_id).ok_or(AppError::NotFound)?;
        let list = &mut rule.trigger_mut(kind).sounds;
        let index = list.iter().position(|s| s.id == sound_id).ok_or(AppError::NotFound)?;
        let removed = list.remove(index);
        Ok((rule.clone(), removed))
    })?;
    sounds::remove(state.store.sounds_dir(), &removed);
    state.engine().forget_rule(&rule.id);
    Ok(rule)
}

#[tauri::command]
pub fn reorder_sound(
    handle: AppHandle,
    state: State<AppState>,
    rule_id: String,
    kind: TriggerKind,
    sound_id: String,
    to_index: usize,
) -> AppResult<Rule> {
    let rule = state.update(&handle, |c| {
        let rule = c.rule_mut(&rule_id).ok_or(AppError::NotFound)?;
        let list = &mut rule.trigger_mut(kind).sounds;
        let from = list.iter().position(|s| s.id == sound_id).ok_or(AppError::NotFound)?;
        let sound = list.remove(from);
        list.insert(to_index.min(list.len()), sound);
        Ok(rule.clone())
    })?;
    state.engine().forget_rule(&rule.id);
    Ok(rule)
}

#[tauri::command]
pub fn preview_sound(state: State<AppState>, rule_id: String, kind: TriggerKind, sound_id: String) -> AppResult<()> {
    let config = state.snapshot();
    let rule = config.rule(&rule_id).ok_or(AppError::NotFound)?;
    let sound = rule.trigger(kind).sounds.iter().find(|s| s.id == sound_id).ok_or(AppError::NotFound)?;
    let path = state.store.sound_path(&sound.file);
    if !path.exists() {
        return Err(AppError::NotFound);
    }
    state.audio.preview(path, rule.volume);
    Ok(())
}

#[tauri::command]
pub fn stop_preview(state: State<AppState>) {
    state.audio.stop_preview();
}

#[tauri::command]
pub fn accessibility_status(prompt: bool) -> bool {
    app_watch::window::accessibility_trusted(prompt)
}

#[tauri::command]
pub fn quit_app(handle: AppHandle) {
    handle.exit(0);
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

#[tauri::command(async)]
pub fn app_icon(app: AppIdentity) -> Option<String> {
    use base64::Engine;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let key = app.key();
    if let Some(hit) = cache.lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return hit.clone();
    }
    let url = app_watch::icon::app_icon_png(&app, 128).map(|png| {
        format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png))
    });
    cache.lock().unwrap_or_else(|e| e.into_inner()).insert(key, url.clone());
    url
}