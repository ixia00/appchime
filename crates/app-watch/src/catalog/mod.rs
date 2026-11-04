mod mac_bundle;
mod win_shortcut;

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use crate::identity::{path_key, AppIdentity};
use crate::process::ProcessScanner;

pub use mac_bundle::{bundle_identity, bundle_root_of_executable, is_background_bundle};

pub fn running_apps() -> Vec<AppIdentity> {
    let own_exe = env::current_exe().ok().map(|p| path_key(&p));
    let mut scanner = ProcessScanner::new();
    let apps = scanner
        .scan()
        .into_iter()
        .filter_map(|p| p.exe_path)
        .filter(|exe| Some(path_key(exe)) != own_exe)
        .filter_map(|exe| identity_for_running_exe(&exe));
    finish(apps)
}

pub fn installed_apps() -> Vec<AppIdentity> {
    let apps: Vec<AppIdentity> = if cfg!(target_os = "macos") {
        mac_bundle::scan_dirs(&mac_app_dirs())
    } else if cfg!(target_os = "windows") {
        win_shortcut::scan_dirs(&windows_start_menu_dirs())
    } else {
        Vec::new()
    };
    finish(apps.into_iter())
}

pub fn identity_from_path(path: &Path) -> Option<AppIdentity> {
    if path.is_dir() {
        return if is_app_bundle(path) { bundle_identity(path) } else { None };
    }
    if !path.is_file() {
        return None;
    }
    if cfg!(target_os = "macos") {
        if let Some(root) = bundle_root_of_executable(path) {
            return bundle_identity(&root);
        }
    }
    Some(plain_identity(path))
}

fn identity_for_running_exe(exe: &Path) -> Option<AppIdentity> {
    if cfg!(target_os = "macos") {
        let root = bundle_root_of_executable(exe)?;
        let root_str = root.to_string_lossy();
        if root_str.starts_with("/System/Library/") || root_str.starts_with("/Library/Apple/") {
            return None;
        }
        if is_background_bundle(&root) {
            return None;
        }
        let identity = bundle_identity(&root)?;
        (path_key(&identity.exe_path) == path_key(exe)).then_some(identity)
    } else if cfg!(target_os = "windows") {
        let system_root = env::var_os("SystemRoot").map(|r| path_key(Path::new(&r)));
        let key = path_key(exe);
        if system_root.is_some_and(|root| key.starts_with(&root)) {
            return None;
        }
        Some(plain_identity(exe))
    } else {
        let s = exe.to_string_lossy();
        if s.starts_with("/usr/lib") || s.starts_with("/usr/libexec") || s.starts_with("/usr/sbin") {
            return None;
        }
        Some(plain_identity(exe))
    }
}

fn plain_identity(exe: &Path) -> AppIdentity {
    let name = exe
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| exe.to_string_lossy().into_owned());
    AppIdentity { name, exe_path: exe.to_path_buf(), bundle_path: None, bundle_id: None }
}

fn is_app_bundle(path: &Path) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("app"))
}

fn finish(apps: impl Iterator<Item = AppIdentity>) -> Vec<AppIdentity> {
    let mut seen = HashSet::new();
    let mut list: Vec<AppIdentity> = apps.filter(|a| seen.insert(a.key())).collect();
    list.sort_by_key(|a| a.name.to_lowercase());
    list
}

fn mac_app_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ];
    if let Some(home) = env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }
    dirs
}

fn windows_start_menu_dirs() -> Vec<PathBuf> {
    const SUFFIX: &str = r"Microsoft\Windows\Start Menu\Programs";
    ["ProgramData", "APPDATA"]
        .iter()
        .filter_map(env::var_os)
        .map(|base| PathBuf::from(base).join(SUFFIX))
        .collect()
}
