use tauri::AppHandle;

use crate::models::Break;

#[cfg(target_os = "macos")]
mod platform {
    use super::{AppHandle, Break};

    // macOS's legacy `NSUserNotification` API (what `tauri-plugin-notification`
    // uses by default via `notify-rust`) has been silently non-functional in
    // testing here — permission can show as granted yet nothing ever
    // displays. `UNUserNotificationCenter`, the modern replacement, requires
    // the process to be a real signed `.app` bundle (it crashes without one,
    // hence the `check_bundle` guard baked into every call below), which is
    // exactly the gap: it only works from a built app, never from
    // `cargo tauri dev`'s raw binary.
    pub fn request_permission(_app: &AppHandle) {
        match mac_usernotifications::blocking::request_auth() {
            Ok(true) => log::info!("notification permission granted"),
            Ok(false) => log::warn!("notification permission denied by the user"),
            Err(e) => log::warn!(
                "could not request notification permission ({e}) — expected when running via \
                 `cargo tauri dev`, since UNUserNotificationCenter requires a real .app bundle; \
                 use `cargo tauri build` to test notification-style breaks"
            ),
        }
    }

    pub fn send_break_notification(app: &AppHandle, b: &Break) {
        // `send_blocking` only avoids hanging forever off the main thread by
        // refusing to run at all unless it can first confirm (via a racy
        // heuristic — `CFRunLoop::main().is_waiting()`) that the main
        // thread's run loop is idling *right now*. That check can catch it
        // mid-cycle and fail spuriously even though the app is running
        // completely normally. Called from the main thread there's no
        // heuristic involved: it drives its own run-loop pump directly, so
        // hop over there via Tauri's main-thread dispatch instead of calling
        // this from the scheduler's background tokio task.
        let title = b.break_type.label();
        let message = b.message.clone();
        let type_label = format!("{:?}", b.break_type);

        let dispatch_result = app.run_on_main_thread(move || {
            let result = mac_usernotifications::Notification::new()
                .title(title)
                .message(message)
                .send_blocking();

            match result {
                Ok(_) => log::info!("break notification shown for {type_label}"),
                Err(e) => log::error!("failed to show break notification: {e}"),
            }
        });

        if let Err(e) = dispatch_result {
            log::error!("failed to schedule break notification on main thread: {e}");
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::{AppHandle, Break};
    use tauri_plugin_notification::NotificationExt;

    pub fn request_permission(app: &AppHandle) {
        let _ = app.notification().request_permission();
    }

    pub fn send_break_notification(app: &AppHandle, b: &Break) {
        let result = app
            .notification()
            .builder()
            .title(b.break_type.label())
            .body(&b.message)
            .show();

        match result {
            Ok(()) => log::info!("break notification shown for {:?}", b.break_type),
            Err(e) => log::error!("failed to show break notification: {e}"),
        }
    }
}

pub use platform::*;
