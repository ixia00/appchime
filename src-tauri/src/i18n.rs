use crate::config::Language;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ko,
    En,
    Ja,
}

pub fn resolve(language: Language) -> Lang {
    match language {
        Language::Ko => Lang::Ko,
        Language::En => Lang::En,
        Language::Ja => Lang::Ja,
        Language::System => {
            let locale = sys_locale::get_locale().unwrap_or_default().to_lowercase();
            if locale.starts_with("ko") {
                Lang::Ko
            } else if locale.starts_with("ja") {
                Lang::Ja
            } else {
                Lang::En
            }
        }
    }
}

pub struct TrayTexts {
    pub status: String,
    pub sound_on: &'static str,
    pub open: &'static str,
    pub quit: &'static str,
    pub tooltip: String,
}

pub fn tray_texts(lang: Lang, total: usize, active: usize, sound_enabled: bool) -> TrayTexts {
    match lang {
        Lang::Ko => TrayTexts {
            status: format!("앱 {total}개 등록 · {active}개 사용 중"),
            sound_on: "소리 켜짐",
            open: "설정 열기…",
            quit: "AppChime 종료",
            tooltip: format!("AppChime — 소리 {}", if sound_enabled { "켜짐" } else { "꺼짐" }),
        },
        Lang::Ja => TrayTexts {
            status: format!("登録 {total} 件 · 使用中 {active} 件"),
            sound_on: "サウンド オン",
            open: "設定を開く…",
            quit: "AppChime を終了",
            tooltip: format!("AppChime — サウンド {}", if sound_enabled { "オン" } else { "オフ" }),
        },
        Lang::En => TrayTexts {
            status: format!("{total} apps · {active} active"),
            sound_on: "Sound On",
            open: "Open Settings…",
            quit: "Quit AppChime",
            tooltip: format!("AppChime — Sound {}", if sound_enabled { "on" } else { "off" }),
        },
    }
}
