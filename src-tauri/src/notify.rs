use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::models::Break;

pub fn send_break_notification(app: &AppHandle, b: &Break) {
    let result = app
        .notification()
        .builder()
        .title(b.break_type.label())
        .body(&b.message)
        .show();

    if let Err(e) = result {
        log::error!("failed to show break notification: {e}");
    }
}
