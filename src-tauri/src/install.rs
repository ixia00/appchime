use std::path::{Path, PathBuf};
use std::process::Command;

pub fn relocate_if_needed() -> bool {
    if !cfg!(target_os = "macos") || cfg!(debug_assertions) {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else { return false };
    if !exe.starts_with("/Volumes/") {
        return false;
    }
    let Some(bundle) = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")) else {
        return false;
    };
    let Some(name) = bundle.file_name() else { return false };
    let volume: PathBuf = exe.components().take(3).collect();

    for dir in target_dirs() {
        let dest = dir.join(name);
        if copy_bundle(bundle, &dest) {
            let script = format!(
                "sleep 1; open \"{}\"; hdiutil detach \"{}\" -quiet",
                dest.display(),
                volume.display()
            );
            let _ = Command::new("sh").arg("-c").arg(script).spawn();
            return true;
        }
    }
    false
}

fn target_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/Applications")];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }
    dirs
}

fn copy_bundle(src: &Path, dest: &Path) -> bool {
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if dest.exists() && std::fs::remove_dir_all(dest).is_err() {
        return false;
    }
    Command::new("ditto").arg(src).arg(dest).status().map(|s| s.success()).unwrap_or(false)
}