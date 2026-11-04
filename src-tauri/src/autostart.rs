
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub const LAUNCH_ARG: &str = "--autostart";

pub fn launched_by_autostart() -> bool {
    std::env::args().any(|a| a == LAUNCH_ARG)
}

pub fn apply(handle: &AppHandle, enabled: bool) {
    if cfg!(debug_assertions) {
        log::info!("개발 모드: 자동 실행 설정({enabled})은 실제로 적용하지 않습니다");
        return;
    }
    let launcher = handle.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    if let Err(e) = result {
        log::error!("자동 실행 설정을 바꾸지 못했습니다: {e}");
    }
}

pub fn sync(handle: &AppHandle, wanted: bool) {
    if cfg!(debug_assertions) {
        return;
    }
    match handle.autolaunch().is_enabled() {
        Ok(actual) if actual == wanted => {}
        Ok(_) => apply(handle, wanted),
        Err(e) => log::warn!("자동 실행 상태를 확인하지 못했습니다: {e}"),
    }
}
