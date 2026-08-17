use chrono::Local;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::Settings;
use crate::persistence;

pub const SETTINGS_CHANGED_EVENT: &str = "settings-changed";

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.data.lock().unwrap().settings.clone()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    settings: Settings,
) -> Result<(), AppError> {
    let mut data = state.data.lock().unwrap();
    data.settings = settings;
    persistence::save_app_data(&app, &data)?;
    crate::tray::refresh_menu(&app, data.settings.paused);
    crate::tray::refresh_label(&app, &data, Local::now().naive_local());
    let _ = app.emit(SETTINGS_CHANGED_EVENT, data.settings.clone());
    Ok(())
}

/// Called from the tray menu's Pause/Resume item, which has no direct
/// access to the frontend's `update_settings` invoke path. Refreshes the
/// tray immediately (rather than waiting for the next scheduler tick) and
/// notifies any open settings window so its "Breaks are active" toggle
/// doesn't go stale.
pub fn toggle_paused_from_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut data = state.data.lock().unwrap();
    data.settings.paused = !data.settings.paused;
    if let Err(e) = persistence::save_app_data(app, &data) {
        log::error!("failed to persist settings: {e}");
    }
    crate::tray::refresh_menu(app, data.settings.paused);
    crate::tray::refresh_label(app, &data, Local::now().naive_local());
    let _ = app.emit(SETTINGS_CHANGED_EVENT, data.settings.clone());
}
