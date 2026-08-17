pub mod break_actions;
pub mod breaks;
pub mod files;
pub mod settings;

use tauri::{AppHandle, Manager};

pub fn open_settings_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
