
use std::fs;
use std::path::Path;

use crate::audio;
use crate::config::Sound;
use crate::error::{AppError, AppResult};

pub const SUPPORTED_EXTENSIONS: [&str; 6] = ["mp3", "wav", "ogg", "flac", "m4a", "aac"];

const MAX_FILE_BYTES: u64 = 50 * 1024 * 1024;

pub fn import(src: &Path, sounds_dir: &Path) -> AppResult<Sound> {
    let display_name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| src.to_string_lossy().into_owned());

    let ext = src
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .filter(|e| SUPPORTED_EXTENSIONS.contains(&e.as_str()))
        .ok_or_else(|| AppError::UnsupportedAudio(display_name.clone()))?;

    if fs::metadata(src)?.len() > MAX_FILE_BYTES {
        return Err(AppError::FileTooLarge(display_name));
    }

    let duration = audio::probe(src).map_err(|_| AppError::UnsupportedAudio(display_name.clone()))?;

    let id = uuid::Uuid::new_v4().to_string();
    let file = format!("{id}.{ext}");
    fs::create_dir_all(sounds_dir)?;
    fs::copy(src, sounds_dir.join(&file))?;

    Ok(Sound {
        id,
        name: display_name,
        file,
        duration_ms: duration.map(|d| d.as_millis() as u64),
    })
}

pub fn remove(sounds_dir: &Path, sound: &Sound) {
    if sound.file.contains(['/', '\\']) || sound.file.contains("..") {
        log::warn!("잘못된 소리 파일 이름을 건너뜁니다: {}", sound.file);
        return;
    }
    match fs::remove_file(sounds_dir.join(&sound.file)) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => log::warn!("소리 파일을 지우지 못했습니다 {}: {e}", sound.file),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_extension() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("note.txt");
        fs::write(&src, "hi").unwrap();
        assert!(matches!(import(&src, dir.path()), Err(AppError::UnsupportedAudio(_))));
    }

    #[test]
    fn rejects_fake_audio() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("fake.mp3");
        fs::write(&src, "definitely not audio").unwrap();
        assert!(matches!(import(&src, dir.path()), Err(AppError::UnsupportedAudio(_))));
    }

    #[test]
    fn imports_real_wav() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("beep.wav");
        fs::write(&src, tiny_wav()).unwrap();
        let sounds = dir.path().join("sounds");
        let sound = import(&src, &sounds).unwrap();
        assert_eq!(sound.name, "beep.wav");
        assert!(sounds.join(&sound.file).exists());
        remove(&sounds, &sound);
        assert!(!sounds.join(&sound.file).exists());
    }

    fn tiny_wav() -> Vec<u8> {
        let rate: u32 = 8000;
        let samples: u32 = rate / 10;
        let data_len = samples * 2;
        let mut v = Vec::new();
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&(36 + data_len).to_le_bytes());
        v.extend_from_slice(b"WAVEfmt ");
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&rate.to_le_bytes());
        v.extend_from_slice(&(rate * 2).to_le_bytes());
        v.extend_from_slice(&2u16.to_le_bytes());
        v.extend_from_slice(&16u16.to_le_bytes());
        v.extend_from_slice(b"data");
        v.extend_from_slice(&data_len.to_le_bytes());
        v.resize(v.len() + data_len as usize, 0);
        v
    }
}
