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
    drop(sched);
    crate::overlay::close_overlay_window(&app);
    Ok(())
}
