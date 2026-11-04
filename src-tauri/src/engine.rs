use std::collections::HashMap;
use std::time::{Duration, Instant};

use app_watch::{AppEvent, AppEventKind};

use crate::config::{Config, LaunchTiming, PlayMode, Trigger, TriggerKind};

#[derive(Debug, Clone, PartialEq)]
pub struct PlayRequest {
    pub file: String,
    pub volume: f32,
    pub wait_for_window: bool,
    pub delay_ms: u32,
}

type Slot = (String, TriggerKind);

#[derive(Default)]
pub struct Engine {
    last_played: HashMap<Slot, Instant>,
    cursors: HashMap<Slot, usize>,
}

impl Engine {
    pub fn plan(&mut self, config: &Config, event: &AppEvent, now: Instant) -> Vec<PlayRequest> {
        if !config.settings.sound_enabled {
            return Vec::new();
        }
        let kind = match event.kind {
            AppEventKind::Launched => TriggerKind::Launch,
            AppEventKind::Quit => TriggerKind::Quit,
        };
        let dedupe = Duration::from_secs(config.settings.dedupe_seconds as u64);

        let mut out = Vec::new();
        for rule in config.rules.iter().filter(|r| r.enabled && r.app.matches(&event.process)) {
            let trigger = rule.trigger(kind);
            if !trigger.enabled || trigger.sounds.is_empty() {
                continue;
            }
            let slot = (rule.id.clone(), kind);
            if let Some(last) = self.last_played.get(&slot) {
                if !dedupe.is_zero() && now.duration_since(*last) < dedupe {
                    continue;
                }
            }
            let index = self.pick(&slot, trigger);
            self.last_played.insert(slot, now);
            out.push(PlayRequest {
                file: trigger.sounds[index].file.clone(),
                volume: rule.volume,
                wait_for_window: kind == TriggerKind::Launch && trigger.timing == LaunchTiming::MainWindow,
                delay_ms: trigger.delay_ms,
            });
        }
        out
    }

    pub fn forget_rule(&mut self, rule_id: &str) {
        self.last_played.retain(|(id, _), _| id != rule_id);
        self.cursors.retain(|(id, _), _| id != rule_id);
    }

    fn pick(&mut self, slot: &Slot, trigger: &Trigger) -> usize {
        let len = trigger.sounds.len();
        match trigger.mode {
            PlayMode::Fixed => 0,
            PlayMode::Sequential => {
                let next = self.cursors.get(slot).copied().unwrap_or(0) % len;
                self.cursors.insert(slot.clone(), next + 1);
                next
            }
            PlayMode::Random => {
                let previous = self.cursors.get(slot).copied();
                let index = match previous {
                    Some(prev) if len > 1 && prev < len => {
                        let i = fastrand::usize(..len - 1);
                        if i >= prev { i + 1 } else { i }
                    }
                    _ => fastrand::usize(..len),
                };
                self.cursors.insert(slot.clone(), index);
                index
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Rule, Sound};
    use app_watch::{AppIdentity, ProcessInfo};

    fn setup(mode: PlayMode, n: usize) -> Config {
        let mut rule = Rule::new(AppIdentity {
            name: "Editor".into(),
            exe_path: "/apps/Editor".into(),
            bundle_path: None,
            bundle_id: None,
        });
        rule.on_launch.mode = mode;
        rule.on_launch.sounds = (0..n)
            .map(|i| Sound { id: i.to_string(), name: format!("{i}.wav"), file: format!("{i}.wav"), duration_ms: None })
            .collect();
        let mut config = Config::default();
        config.settings.dedupe_seconds = 0;
        config.rules.push(rule);
        config
    }

    fn launch() -> AppEvent {
        AppEvent {
            kind: AppEventKind::Launched,
            process: ProcessInfo { pid: 1, name: "Editor".into(), exe_path: Some("/apps/Editor".into()) },
        }
    }

    #[test]
    fn sequential_cycles() {
        let config = setup(PlayMode::Sequential, 3);
        let mut e = Engine::default();
        let files: Vec<String> =
            (0..4).map(|_| e.plan(&config, &launch(), Instant::now())[0].file.clone()).collect();
        assert_eq!(files, ["0.wav", "1.wav", "2.wav", "0.wav"]);
    }

    #[test]
    fn random_never_repeats_back_to_back() {
        let config = setup(PlayMode::Random, 2);
        let mut e = Engine::default();
        let mut last = String::new();
        for _ in 0..50 {
            let f = e.plan(&config, &launch(), Instant::now())[0].file.clone();
            assert_ne!(f, last);
            last = f;
        }
    }

    #[test]
    fn dedupe_blocks_quick_repeat() {
        let mut config = setup(PlayMode::Fixed, 1);
        config.settings.dedupe_seconds = 5;
        let mut e = Engine::default();
        let t0 = Instant::now();
        assert_eq!(e.plan(&config, &launch(), t0).len(), 1);
        assert_eq!(e.plan(&config, &launch(), t0 + Duration::from_secs(2)).len(), 0);
        assert_eq!(e.plan(&config, &launch(), t0 + Duration::from_secs(6)).len(), 1);
    }

    #[test]
    fn muted_or_disabled_plays_nothing() {
        let mut config = setup(PlayMode::Fixed, 1);
        config.settings.sound_enabled = false;
        assert!(Engine::default().plan(&config, &launch(), Instant::now()).is_empty());

        let mut config = setup(PlayMode::Fixed, 1);
        config.rules[0].on_launch.enabled = false;
        assert!(Engine::default().plan(&config, &launch(), Instant::now()).is_empty());
    }

    #[test]
    fn quit_uses_quit_trigger() {
        let config = setup(PlayMode::Fixed, 1);
        let mut event = launch();
        event.kind = AppEventKind::Quit;
        assert!(Engine::default().plan(&config, &event, Instant::now()).is_empty());
    }
}
