use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::identity::AppIdentity;

pub(crate) fn scan_dirs(dirs: &[PathBuf]) -> Vec<AppIdentity> {
    let mut found = Vec::new();
    for dir in dirs {
        scan_dir(dir, 1, &mut found);
    }
    found
}

fn scan_dir(dir: &Path, depth_left: u8, out: &mut Vec<AppIdentity>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("app")) {
            if is_background_bundle(&path) {
                continue;
            }
            if let Some(identity) = bundle_identity(&path) {
                out.push(identity);
            }
        } else if depth_left > 0 {
            scan_dir(&path, depth_left - 1, out);
        }
    }
}

pub fn is_background_bundle(bundle: &Path) -> bool {
    let Ok(info) = plist::Value::from_file(bundle.join("Contents").join("Info.plist")) else {
        return false;
    };
    let Some(dict) = info.as_dictionary() else { return false };
    ["LSUIElement", "LSBackgroundOnly"].iter().any(|key| match dict.get(*key) {
        Some(plist::Value::Boolean(b)) => *b,
        Some(plist::Value::String(s)) => s == "1" || s.eq_ignore_ascii_case("true"),
        Some(plist::Value::Integer(i)) => i.as_signed() == Some(1),
        _ => false,
    })
}

pub fn bundle_identity(bundle: &Path) -> Option<AppIdentity> {
    let info = plist::Value::from_file(bundle.join("Contents").join("Info.plist")).ok()?;
    let dict = info.as_dictionary()?;
    let get = |key: &str| dict.get(key).and_then(|v| v.as_string()).filter(|s| !s.trim().is_empty());

    let executable = get("CFBundleExecutable")?;
    let name = get("CFBundleDisplayName")
        .or_else(|| get("CFBundleName"))
        .map(str::to_owned)
        .or_else(|| bundle.file_stem().map(|s| s.to_string_lossy().into_owned()))?;

    Some(AppIdentity {
        name,
        exe_path: bundle.join("Contents").join("MacOS").join(executable),
        bundle_path: Some(bundle.to_path_buf()),
        bundle_id: get("CFBundleIdentifier").map(str::to_owned),
    })
}

pub fn bundle_root_of_executable(exe: &Path) -> Option<PathBuf> {
    let parts: Vec<Component> = exe.components().collect();
    let is_app = |c: &Component| {
        Path::new(c.as_os_str()).extension().is_some_and(|e| e.eq_ignore_ascii_case("app"))
    };
    let first_app = parts.iter().position(is_app)?;

    let tail: Vec<String> = parts[first_app + 1..]
        .iter()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if tail.len() != 3 || tail[0] != "Contents" || tail[1] != "MacOS" {
        return None;
    }
    Some(parts[..=first_app].iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_bundle_root() {
        let exe = Path::new("/Applications/VOCALOID6 Editor.app/Contents/MacOS/VOCALOID6 Editor");
        assert_eq!(
            bundle_root_of_executable(exe),
            Some(PathBuf::from("/Applications/VOCALOID6 Editor.app"))
        );
    }

    #[test]
    fn rejects_nested_helper_app() {
        let exe = Path::new(
            "/Applications/Foo.app/Contents/Frameworks/Foo Helper.app/Contents/MacOS/Foo Helper",
        );
        assert_eq!(bundle_root_of_executable(exe), None);
    }

    #[test]
    fn detects_background_only_app() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("Helper.app");
        fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        fs::write(
            bundle.join("Contents/Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>Helper</string>
<key>LSUIElement</key><true/>
</dict></plist>"#,
        )
        .unwrap();
        assert!(is_background_bundle(&bundle));
        assert!(scan_dirs(&[dir.path().to_path_buf()]).is_empty());
    }

    #[test]
    fn reads_info_plist() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("Sample.app");
        fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        fs::write(
            bundle.join("Contents/Info.plist"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>SampleBin</string>
<key>CFBundleIdentifier</key><string>com.example.sample</string>
<key>CFBundleName</key><string>Sample</string>
</dict></plist>"#,
        )
        .unwrap();

        let id = bundle_identity(&bundle).unwrap();
        assert_eq!(id.name, "Sample");
        assert_eq!(id.bundle_id.as_deref(), Some("com.example.sample"));
        assert_eq!(id.exe_path, bundle.join("Contents/MacOS/SampleBin"));

        let scanned = scan_dirs(&[dir.path().to_path_buf()]);
        assert_eq!(scanned.len(), 1);
    }
}
