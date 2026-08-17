pub mod label;

use chrono::NaiveDateTime;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Wry};

use crate::models::AppData;
use crate::scheduler::trigger::next_occurrence;

const TRAY_ID: &str = "main-tray";

// A simple monochrome glyph (alpha-only shape, opaque black), stored as raw
// RGBA rather than the app's full-color logo — marked as a template image
// below so macOS tints it to match the menu bar (white on dark, black on
// light) instead of showing distracting fixed colors. Raw RGBA (rather than
// PNG) avoids needing the "image-png" cargo feature just to decode one
// small icon at startup.
const TRAY_ICON_SIZE: u32 = 44;
const TRAY_ICON_RGBA: &[u8] = include_bytes!("../../icons/tray-icon.rgba");

pub fn build_tray(app: &AppHandle, paused: bool) -> tauri::Result<TrayIcon<Wry>> {
    let menu = build_menu(app, paused)?;
    let icon = Image::new(TRAY_ICON_RGBA, TRAY_ICON_SIZE, TRAY_ICON_SIZE);

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .title("--")
        .on_menu_event(|app, event| handle_menu_event(app, event.id.as_ref()))
        .build(app)
}

fn build_menu(app: &AppHandle, paused: bool) -> tauri::Result<Menu<Wry>> {
    let toggle_label = if paused {
        "Resume Breaks"
    } else {
        "Pause Breaks"
    };
    let toggle = MenuItem::with_id(app, "toggle_pause", toggle_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "open_settings", "Open Settings…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    Menu::with_items(app, &[&toggle, &settings, &separator, &quit])
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "toggle_pause" => crate::commands::settings::toggle_paused_from_tray(app),
        "open_settings" => crate::commands::open_settings_window(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

pub fn refresh_menu(app: &AppHandle, paused: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Ok(menu) = build_menu(app, paused) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

pub fn refresh_label(app: &AppHandle, data: &AppData, now: NaiveDateTime) {
    let text = if data.settings.paused {
        "Paused".to_string()
    } else {
        match next_occurrence(now, &data.breaks) {
            Some(next) => {
                let minutes = (next - now).num_minutes().max(0);
                label::format_remaining(minutes)
            }
            None => "--".to_string(),
        }
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_title(Some(text));
    }
}
