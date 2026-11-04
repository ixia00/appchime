use std::fs;
use std::path::{Path, PathBuf};

use crate::identity::AppIdentity;

pub(crate) fn scan_dirs(dirs: &[PathBuf]) -> Vec<AppIdentity> {
    let mut found = Vec::new();
    for dir in dirs {
        scan_dir(dir, 4, &mut found);
    }
    found
}

fn scan_dir(dir: &Path, depth_left: u8, out: &mut Vec<AppIdentity>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth_left > 0 {
                scan_dir(&path, depth_left - 1, out);
            }
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) {
            if let Some(identity) = shortcut_identity(&path) {
                out.push(identity);
            }
        }
    }
}

fn shortcut_identity(lnk_path: &Path) -> Option<AppIdentity> {
    let name = lnk_path.file_stem()?.to_string_lossy().into_owned();
    if is_noise(&name) {
        return None;
    }
    let target = resolve_target(lnk_path)?;
    let is_exe = target.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    let target_name = target.file_stem()?.to_string_lossy().to_lowercase();
    if !is_exe || is_noise(&target_name) {
        return None;
    }
    Some(AppIdentity { name, exe_path: target, bundle_path: None, bundle_id: None })
}

fn resolve_target(lnk_path: &Path) -> Option<PathBuf> {
    let link = lnk::ShellLink::open(lnk_path, lnk::encoding::WINDOWS_1252).ok()?;
    let info = link.link_info().as_ref()?;
    let base = info.local_base_path()?;
    let suffix = info.common_path_suffix();
    let full = format!("{base}{suffix}");
    (!full.is_empty()).then(|| PathBuf::from(full))
}

fn is_noise(name: &str) -> bool {
    const WORDS: [&str; 8] =
        ["uninstall", "unins0", "제거", "アンインストール", "readme", "help", "도움말", "ヘルプ"];
    let lower = name.to_lowercase();
    WORDS.iter().any(|w| lower.contains(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_uninstallers() {
        assert!(is_noise("Uninstall VOCALOID6"));
        assert!(is_noise("unins000"));
        assert!(!is_noise("VOCALOID6 Editor"));
    }
}
