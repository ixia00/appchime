mod store;

pub use store::Store;

use app_watch::AppIdentity;
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "current_version")]
    pub version: u32,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self { version: CONFIG_VERSION, settings: Settings::default(), rules: Vec::new() }
    }
}

impl Config {
    pub fn rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn rule_mut(&mut self, id: &str) -> Option<&mut Rule> {
        self.rules.iter_mut().find(|r| r.id == id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub sound_enabled: bool,
    pub launch_at_login: bool,
    pub toggle_shortcut: Option<String>,
    pub dedupe_seconds: u32,
    pub language: Language,
    pub theme: Theme,
    pub accent: Accent,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            launch_at_login: true,
            toggle_shortcut: None,
            dedupe_seconds: 5,
            language: Language::System,
            theme: Theme::System,
            accent: Accent::Blue,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    System,
    Ko,
    En,
    Ja,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    #[default]
    Blue,
    Purple,
    Pink,
    Orange,
    Green,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub app: AppIdentity,
    pub enabled: bool,
    pub volume: f32,
    pub on_launch: Trigger,
    pub on_quit: Trigger,
}

impl Rule {
    pub fn new(app: AppIdentity) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            app,
            enabled: true,
            volume: 0.8,
            on_launch: Trigger { enabled: true, ..Trigger::default() },
            on_quit: Trigger::default(),
        }
    }

    pub fn trigger(&self, kind: TriggerKind) -> &Trigger {
        match kind {
            TriggerKind::Launch => &self.on_launch,
            TriggerKind::Quit => &self.on_quit,
        }
    }

    pub fn trigger_mut(&mut self, kind: TriggerKind) -> &mut Trigger {
        match kind {
            TriggerKind::Launch => &mut self.on_launch,
            TriggerKind::Quit => &mut self.on_quit,
        }
    }

    pub fn all_sounds(&self) -> impl Iterator<Item = &Sound> {
        self.on_launch.sounds.iter().chain(self.on_quit.sounds.iter())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Trigger {
    pub enabled: bool,
    pub mode: PlayMode,
    pub sounds: Vec<Sound>,
    pub timing: LaunchTiming,
    pub delay_ms: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum LaunchTiming {
    #[default]
    MainWindow,
    Process,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlayMode {
    #[default]
    Fixed,
    Random,
    Sequential,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TriggerKind {
    Launch,
    Quit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    pub id: String,
    pub name: String,
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

fn current_version() -> u32 {
    CONFIG_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_files_without_new_fields_still_load() {
        let json = r#"{ "rules": [] }"#;
        let c: Config = serde_json::from_str(json).unwrap();
        assert_eq!(c.settings, Settings::default());
    }

    #[test]
    fn roundtrip() {
        let mut c = Config::default();
        c.rules.push(Rule::new(AppIdentity {
            name: "A".into(),
            exe_path: "/a".into(),
            bundle_path: None,
            bundle_id: None,
        }));
        let s = serde_json::to_string(&c).unwrap();
        assert!(s.contains("\"onLaunch\""));
        let back: Config = serde_json::from_str(&s).unwrap();
        assert_eq!(back, c);
    }
}
