use chrono::{Duration, Local};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::persistence;

#[tauri::command]
pub fn postpone_break(
    app: AppHandle,
    state: State<AppState>,
    break_id: Uuid,
    minutes: i64,
) -> Result<(), AppError> {
    let mut sched = state.scheduler_state.lock().unwrap();
    let fire_at = Local::now().naive_local() + Duration::minutes(minutes);
    sched.postponed.insert(break_id, fire_at);
    persistence::save_state(&app, &sched)?;
    refresh_tray_label(&app, &state, &sched);
    drop(sched);
    crate::overlay::close_overlay_window(&app);
    Ok(())
}

#[tauri::command]
pub fn cancel_break(
    app: AppHandle,
    state: State<AppState>,
    break_id: Uuid,
) -> Result<(), AppError> {
    let mut sched = state.scheduler_state.lock().unwrap();
    let today = Local::now().naive_local().date();
    sched.skipped_today.insert(break_id, today);
    persistence::save_state(&app, &sched)?;
    refresh_tray_label(&app, &state, &sched);
    drop(sched);
    crate::overlay::close_overlay_window(&app);
    Ok(())
}

/// Postpone/cancel change what the tray's "time until next break" label
/// should show right now — without this, the label would keep showing the
/// pre-postpone countdown until the next scheduler tick (up to 15s later).
fn refresh_tray_label(
    app: &AppHandle,
    state: &AppState,
    sched: &crate::scheduler::state::SchedulerState,
) {
    let data = state.data.lock().unwrap();
    crate::tray::refresh_label(app, &data, sched, Local::now().naive_local());
}
