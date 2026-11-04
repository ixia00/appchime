use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::config::Config;
use crate::i18n::{self, TrayTexts};
use crate::{actions, window};

const TRAY_ID: &str = "main";

const MENU_TOGGLE: &str = "toggle";
const MENU_OPEN: &str = "open";
const MENU_QUIT: &str = "quit";

#[cfg(target_os = "macos")]
mod icons {
    pub const ON: &[u8] = include_bytes!("../icons/tray-template.png");
    pub const OFF: &[u8] = include_bytes!("../icons/tray-template-muted.png");
    pub const TEMPLATE: bool = true;
}
#[cfg(not(target_os = "macos"))]
mod icons {
    pub const ON: &[u8] = include_bytes!("../icons/tray-color.png");
    pub const OFF: &[u8] = include_bytes!("../icons/tray-color-muted.png");
    pub const TEMPLATE: bool = false;
}

struct TrayMenu {
    status: MenuItem<Wry>,
    toggle: CheckMenuItem<Wry>,
    open: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn create(handle: &AppHandle, config: &Config) -> tauri::Result<()> {
    let t = texts(config);
    let enabled = config.settings.sound_enabled;

    let status = MenuItem::with_id(handle, "status", &t.status, false, None::<&str>)?;
    let toggle = CheckMenuItem::with_id(handle, MENU_TOGGLE, t.sound_on, true, enabled, None::<&str>)?;
    let open = MenuItem::with_id(handle, MENU_OPEN, t.open, true, None::<&str>)?;
    let quit = MenuItem::with_id(handle, MENU_QUIT, t.quit, true, None::<&str>)?;
    let menu = Menu::with_items(
        handle,
        &[
            &status,
            &toggle,
            &PredefinedMenuItem::separator(handle)?,
            &open,
            &PredefinedMenuItem::separator(handle)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon(enabled)?)
        .icon_as_template(icons::TEMPLATE)
        .tooltip(&t.tooltip)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_TOGGLE => actions::toggle_sound(app),
            MENU_OPEN => window::show_settings(app),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .build(handle)?;

    handle.manage(TrayMenu { status, toggle, open, quit });
    Ok(())
}

pub fn refresh(handle: &AppHandle, config: &Config) {
    let t = texts(config);
    let enabled = config.settings.sound_enabled;

    if let Some(menu) = handle.try_state::<TrayMenu>() {
        let _ = menu.status.set_text(&t.status);
        let _ = menu.toggle.set_text(t.sound_on);
        let _ = menu.toggle.set_checked(enabled);
        let _ = menu.open.set_text(t.open);
        let _ = menu.quit.set_text(t.quit);
    }
    if let Some(tray) = handle.tray_by_id(TRAY_ID) {
        if let Ok(img) = icon(enabled) {
            let _ = tray.set_icon(Some(img));
            let _ = tray.set_icon_as_template(icons::TEMPLATE);
        }
        let _ = tray.set_tooltip(Some(&t.tooltip));
    }
}

fn texts(config: &Config) -> TrayTexts {
    let total = config.rules.len();
    let active = config.rules.iter().filter(|r| r.enabled).count();
    i18n::tray_texts(i18n::resolve(config.settings.language), total, active, config.settings.sound_enabled)
}

fn icon(enabled: bool) -> tauri::Result<Image<'static>> {
    Image::from_bytes(if enabled { icons::ON } else { icons::OFF })
}
