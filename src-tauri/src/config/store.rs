use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::Config;
use crate::error::AppResult;

const CONFIG_FILE: &str = "config.json";
const SOUNDS_DIR: &str = "sounds";

pub struct Store {
    config_path: PathBuf,
    sounds_dir: PathBuf,
}

pub struct Loaded {
    pub config: Config,
    pub first_run: bool,
}

impl Store {
    pub fn new(config_dir: &Path, data_dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(config_dir)?;
        let sounds_dir = data_dir.join(SOUNDS_DIR);
        fs::create_dir_all(&sounds_dir)?;
        Ok(Self { config_path: config_dir.join(CONFIG_FILE), sounds_dir })
    }

    pub fn sounds_dir(&self) -> &Path {
        &self.sounds_dir
    }

    pub fn sound_path(&self, file: &str) -> PathBuf {
        self.sounds_dir.join(file)
    }

    pub fn load(&self) -> Loaded {
        let text = match fs::read_to_string(&self.config_path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Loaded { config: Config::default(), first_run: true };
            }
            Err(e) => {
                log::error!("설정 파일을 읽지 못했습니다: {e}");
                return Loaded { config: Config::default(), first_run: false };
            }
        };

        match serde_json::from_str::<Config>(&text) {
            Ok(config) => Loaded { config, first_run: false },
            Err(e) => {
                log::error!("설정 파일이 손상되어 백업 후 초기화합니다: {e}");
                let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                let backup = self.config_path.with_extension(format!("json.broken-{stamp}"));
                let _ = fs::rename(&self.config_path, backup);
                Loaded { config: Config::default(), first_run: false }
            }
        }
    }

    pub fn save(&self, config: &Config) -> AppResult<()> {
        let json = serde_json::to_vec_pretty(config)?;
        let tmp = self.config_path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &self.config_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_then_save_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(&dir.path().join("c"), &dir.path().join("d")).unwrap();
        let loaded = store.load();
        assert!(loaded.first_run);

        let mut config = loaded.config;
        config.settings.dedupe_seconds = 9;
        store.save(&config).unwrap();

        let again = store.load();
        assert!(!again.first_run);
        assert_eq!(again.config.settings.dedupe_seconds, 9);
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(&dir.path().join("c"), &dir.path().join("d")).unwrap();
        fs::write(dir.path().join("c").join(CONFIG_FILE), "{ not json").unwrap();
        let loaded = store.load();
        assert!(!loaded.first_run);
        let backups = fs::read_dir(dir.path().join("c"))
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("broken"))
            .count();
        assert_eq!(backups, 1);
    }
}
