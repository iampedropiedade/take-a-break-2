use chrono::Local;
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::Break;
use crate::persistence;

#[tauri::command]
pub fn list_breaks(state: State<AppState>) -> Vec<Break> {
    state.data.lock().unwrap().breaks.clone()
}

#[tauri::command]
pub fn create_break(
    app: AppHandle,
    state: State<AppState>,
    mut new_break: Break,
) -> Result<Break, AppError> {
    new_break.id = Uuid::new_v4();
    let mut data = state.data.lock().unwrap();
    data.breaks.push(new_break.clone());
    persistence::save_app_data(&app, &data)?;
    let sched = state.scheduler_state.lock().unwrap();
    crate::tray::refresh_label(&app, &data, &sched, Local::now().naive_local());
    Ok(new_break)
}

#[tauri::command]
pub fn update_break(
    app: AppHandle,
    state: State<AppState>,
    updated: Break,
) -> Result<(), AppError> {
    let mut data = state.data.lock().unwrap();
    match data.breaks.iter_mut().find(|b| b.id == updated.id) {
        Some(existing) => *existing = updated.clone(),
        None => return Err(AppError::Message("break not found".into())),
    }
    persistence::save_app_data(&app, &data)?;
    drop(data);

    // A break's schedule may have just changed (e.g. its start time), but
    // the scheduler still remembers whether *the old* schedule already fired
    // today — without clearing that, an edited break can silently refuse to
    // fire again until the next day. Clearing it here means an edit always
    // takes effect immediately.
    clear_scheduler_state_for(&app, &state, updated.id)?;
    Ok(())
}

#[tauri::command]
pub fn delete_break(app: AppHandle, state: State<AppState>, id: Uuid) -> Result<(), AppError> {
    let mut data = state.data.lock().unwrap();
    data.breaks.retain(|b| b.id != id);
    persistence::save_app_data(&app, &data)?;
    drop(data);

    clear_scheduler_state_for(&app, &state, id)?;
    Ok(())
}

fn clear_scheduler_state_for(
    app: &AppHandle,
    state: &State<AppState>,
    id: Uuid,
) -> Result<(), AppError> {
    let mut sched = state.scheduler_state.lock().unwrap();
    sched.last_fired.remove(&id);
    sched.postponed.remove(&id);
    sched.skipped_today.remove(&id);
    persistence::save_state(app, &sched)?;
    let data = state.data.lock().unwrap();
    crate::tray::refresh_label(app, &data, &sched, Local::now().naive_local());
    Ok(())
}
