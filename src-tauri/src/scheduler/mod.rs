pub mod state;
pub mod trigger;

use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use chrono::Local;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::call_detection::CallDetector;
use crate::models::{AppData, Break, DisplayMode};
use crate::persistence;

use self::state::SchedulerState;

const TICK_INTERVAL: StdDuration = StdDuration::from_secs(15);
pub const BREAK_TRIGGERED_EVENT: &str = "break-triggered";

#[derive(Clone, Serialize)]
pub struct BreakTriggeredPayload {
    pub break_id: Uuid,
    pub type_label: String,
    pub message: String,
    pub image_filename: Option<String>,
    pub fullscreen: bool,
    pub duration_minutes: u32,
}

/// Spawns the background scheduler loop on Tauri's async runtime. Ticks
/// every `TICK_INTERVAL`, evaluating the (in-memory, shared) break list
/// against scheduler state each time.
pub fn spawn(
    app: AppHandle,
    app_data: Arc<Mutex<AppData>>,
    scheduler_state: Arc<Mutex<SchedulerState>>,
    call_detector: Arc<dyn CallDetector>,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        loop {
            interval.tick().await;
            tick(&app, &app_data, &scheduler_state, call_detector.as_ref());
        }
    });
}

fn tick(
    app: &AppHandle,
    app_data: &Arc<Mutex<AppData>>,
    scheduler_state: &Arc<Mutex<SchedulerState>>,
    call_detector: &dyn CallDetector,
) {
    let data = app_data.lock().unwrap();

    if data.settings.paused {
        let sched = scheduler_state.lock().unwrap();
        crate::tray::refresh_label(app, &data, &sched, Local::now().naive_local());
        return;
    }

    let call_active = data.settings.cancel_on_call && call_detector.is_active();
    let now = Local::now().naive_local();

    let due_ids = {
        let mut sched = scheduler_state.lock().unwrap();
        let due = trigger::compute_due_breaks(now, &data.breaks, &mut sched, call_active);
        if let Err(e) = persistence::save_state(app, &sched) {
            log::error!("failed to persist scheduler state: {e}");
        }
        due
    };

    for id in &due_ids {
        if let Some(b) = data.breaks.iter().find(|b| &b.id == id) {
            fire(app, &data, b);
        }
    }

    let sched = scheduler_state.lock().unwrap();
    crate::tray::refresh_label(app, &data, &sched, now);
}

fn fire(app: &AppHandle, data: &AppData, b: &Break) {
    let fullscreen = matches!(
        b.display_mode.unwrap_or(data.settings.default_display_mode),
        DisplayMode::Fullscreen
    );

    if fullscreen {
        crate::overlay::open_overlay_window(app, b, data.settings.show_on_all_screens);
    } else {
        crate::notify::send_break_notification(app, b);
    }

    let payload = BreakTriggeredPayload {
        break_id: b.id,
        type_label: b.break_type.label(),
        message: b.message.clone(),
        image_filename: b.image_filename.clone(),
        fullscreen,
        duration_minutes: b.duration_minutes,
    };
    let _ = app.emit(BREAK_TRIGGERED_EVENT, payload);
}
