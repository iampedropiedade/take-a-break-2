pub mod images;

use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::AppData;
use crate::scheduler::state::SchedulerState;

const APP_DATA_FILE: &str = "take_a_break.json";
const STATE_FILE: &str = "state.json";

fn config_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_config_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Each Tauri command runs on its own thread, so rapid edits (e.g. toggling
/// a checkbox repeatedly) can call this concurrently for the same target
/// file. A per-call unique tmp filename means concurrent writers never race
/// on the same path — whichever rename lands last simply wins — rather than
/// one thread's `rename` failing with "not found" because another thread
/// already moved the shared tmp file away.
fn write_atomic(path: &Path, contents: &str) -> Result<(), AppError> {
    let tmp_path = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    fs::write(&tmp_path, contents)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn load_app_data(app: &AppHandle) -> Result<AppData, AppError> {
    let path = config_dir(app)?.join(APP_DATA_FILE);
    if !path.exists() {
        return Ok(AppData::default());
    }
    let contents = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save_app_data(app: &AppHandle, data: &AppData) -> Result<(), AppError> {
    let path = config_dir(app)?.join(APP_DATA_FILE);
    let contents = serde_json::to_string_pretty(data)?;
    write_atomic(&path, &contents)
}

pub fn load_state(app: &AppHandle) -> Result<SchedulerState, AppError> {
    let path = config_dir(app)?.join(STATE_FILE);
    if !path.exists() {
        return Ok(SchedulerState::default());
    }
    let contents = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&contents)?)
}

pub fn save_state(app: &AppHandle, state: &SchedulerState) -> Result<(), AppError> {
    let path = config_dir(app)?.join(STATE_FILE);
    let contents = serde_json::to_string_pretty(state)?;
    write_atomic(&path, &contents)
}
