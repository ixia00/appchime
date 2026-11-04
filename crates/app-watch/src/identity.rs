use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::process::ProcessInfo;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppIdentity {
    pub name: String,
    pub exe_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
}

impl AppIdentity {
    pub fn key(&self) -> String {
        path_key(&self.exe_path)
    }
    pub fn matches(&self, process: &ProcessInfo) -> bool {
        match &process.exe_path {
            Some(path) => path_key(path) == self.key(),
            None => file_name_key(&self.exe_path)
                .map(|name| name == normalize(&process.name))
                .unwrap_or(false),
        }
    }
}

pub(crate) fn path_key(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let trimmed = raw.strip_prefix(r"\\?\").unwrap_or(&raw);
    normalize(trimmed)
}

fn file_name_key(path: &Path) -> Option<String> {
    path.file_name().map(|n| normalize(&n.to_string_lossy()))
}

fn normalize(s: &str) -> String {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        s.to_lowercase()
    } else {
        s.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(path: &str) -> AppIdentity {
        AppIdentity {
            name: "Test".into(),
            exe_path: PathBuf::from(path),
            bundle_path: None,
            bundle_id: None,
        }
    }

    #[test]
    fn matches_by_path() {
        let a = app("/Applications/Test.app/Contents/MacOS/Test");
        let p = ProcessInfo {
            pid: 1,
            name: "Test".into(),
            exe_path: Some("/Applications/Test.app/Contents/MacOS/Test".into()),
        };
        assert!(a.matches(&p));
    }

    #[test]
    fn falls_back_to_name_without_path() {
        let a = app("/opt/test/Test");
        let p = ProcessInfo { pid: 1, name: "Test".into(), exe_path: None };
        assert!(a.matches(&p));
        let other = ProcessInfo { pid: 2, name: "Other".into(), exe_path: None };
        assert!(!a.matches(&other));
    }

    #[test]
    fn strips_windows_extended_prefix() {
        assert_eq!(path_key(Path::new(r"\\?\C:\a.exe")), normalize(r"C:\a.exe"));
    }
}
