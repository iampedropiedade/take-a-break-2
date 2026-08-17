use serde_json::json;
use tauri::{AppHandle, Manager, Monitor, WebviewUrl, WebviewWindowBuilder};

use crate::models::Break;

const OVERLAY_LABEL_PREFIX: &str = "overlay";

/// Builds fresh borderless, always-on-top window(s) for a fullscreen break —
/// one per monitor if `show_on_all_screens` is set, otherwise just the
/// primary monitor. Built fresh per trigger (not a static config window) and
/// destroyed — not hidden — once the break ends, is postponed, or is
/// cancelled, so there's never stale content left around.
pub fn open_overlay_window(app: &AppHandle, b: &Break, show_on_all_screens: bool) {
    close_overlay_window(app);

    let payload = json!({
        "breakId": b.id,
        "typeLabel": b.break_type.label(),
        "message": b.message,
        "imageFilename": b.image_filename,
        "durationMinutes": b.duration_minutes,
    });
    let init_script = format!("window.__BREAK__ = {payload};");

    let Some(main_window) = app.get_webview_window("main") else {
        log::error!("no main window handle available to query monitors from");
        return;
    };

    let monitors: Vec<Monitor> = if show_on_all_screens {
        main_window.available_monitors().unwrap_or_default()
    } else {
        main_window
            .primary_monitor()
            .ok()
            .flatten()
            .into_iter()
            .collect()
    };

    if monitors.is_empty() {
        build_window(app, OVERLAY_LABEL_PREFIX.to_string(), &init_script, None);
        return;
    }

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("{OVERLAY_LABEL_PREFIX}-{i}");
        build_window(app, label, &init_script, Some(monitor));
    }
}

fn build_window(app: &AppHandle, label: String, init_script: &str, monitor: Option<&Monitor>) {
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App("overlay.html".into()))
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .initialization_script(init_script);

    builder = match monitor {
        Some(monitor) => {
            // Monitor size/position are physical pixels; the window builder
            // expects logical ones, so this must go through the monitor's
            // own scale factor rather than being passed through as-is (on a
            // Retina/HiDPI display that previously produced a window twice
            // the screen's actual logical size).
            let scale = monitor.scale_factor();
            let logical_size = monitor.size().to_logical::<f64>(scale);
            let logical_position = monitor.position().to_logical::<f64>(scale);
            builder
                .inner_size(logical_size.width, logical_size.height)
                .position(logical_position.x, logical_position.y)
        }
        None => builder.fullscreen(true),
    };

    if let Err(e) = builder.build() {
        log::error!("failed to open break overlay window: {e}");
    }
}

pub fn close_overlay_window(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with(OVERLAY_LABEL_PREFIX) {
            let _ = window.close();
        }
    }
}
