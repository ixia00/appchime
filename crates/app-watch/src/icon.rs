use std::fs::{self, File};
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};

use crate::identity::AppIdentity;

pub fn app_icon_png(app: &AppIdentity, size: u32) -> Option<Vec<u8>> {
    if let Some(bundle) = &app.bundle_path {
        return mac_bundle_icon(bundle, size);
    }
    let is_exe = app.exe_path.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    if is_exe {
        return exe_icon(&app.exe_path, size);
    }
    None
}

fn mac_bundle_icon(bundle: &Path, size: u32) -> Option<Vec<u8>> {
    let path = icns_path(bundle)?;
    let family = icns::IconFamily::read(BufReader::new(File::open(path).ok()?)).ok()?;

    let mut types = family.available_icons();
    types.sort_by_key(|t| t.pixel_width());
    let ordered = types
        .iter()
        .filter(|t| t.pixel_width() >= size)
        .chain(types.iter().rev().filter(|t| t.pixel_width() < size));

    for icon_type in ordered {
        if let Ok(image) = family.get_icon_with_type(*icon_type) {
            let mut png = Vec::new();
            if image.write_png(&mut png).is_ok() {
                return Some(png);
            }
        }
    }
    None
}

fn icns_path(bundle: &Path) -> Option<PathBuf> {
    let resources = bundle.join("Contents").join("Resources");
    let named = plist::Value::from_file(bundle.join("Contents").join("Info.plist"))
        .ok()
        .and_then(|v| {
            v.as_dictionary()?.get("CFBundleIconFile")?.as_string().map(str::to_owned)
        })
        .map(|name| {
            let file = if name.to_lowercase().ends_with(".icns") { name } else { format!("{name}.icns") };
            resources.join(file)
        })
        .filter(|p| p.is_file());

    named.or_else(|| {
        fs::read_dir(&resources).ok()?.flatten().map(|e| e.path()).find(|p| {
            p.extension().is_some_and(|e| e.eq_ignore_ascii_case("icns"))
        })
    })
}

fn exe_icon(exe: &Path, size: u32) -> Option<Vec<u8>> {
    let map = pelite::FileMap::open(exe).ok()?;
    let pe = pelite::PeFile::from_bytes(&map).ok()?;
    let resources = pe.resources().ok()?;

    let (_, group) = resources.icons().filter_map(Result::ok).next()?;
    let mut ico_bytes = Vec::new();
    group.write(&mut ico_bytes).ok()?;
    ico_to_png(&ico_bytes, size)
}

fn ico_to_png(ico_bytes: &[u8], size: u32) -> Option<Vec<u8>> {
    let dir = ico::IconDir::read(Cursor::new(ico_bytes)).ok()?;
    let mut entries: Vec<_> = dir.entries().iter().collect();
    entries.sort_by_key(|e| e.width());
    let entry = entries
        .iter()
        .find(|e| e.width() >= size)
        .or_else(|| entries.last())?;
    let image = entry.decode().ok()?;
    let mut png = Vec::new();
    image.write_png(&mut png).ok()?;
    Some(png)
}