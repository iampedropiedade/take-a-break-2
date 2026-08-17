use std::sync::{Arc, Mutex};

use crate::models::AppData;
use crate::scheduler::state::SchedulerState;

/// Shared runtime state managed by Tauri and reached from command handlers.
/// Plain `std::sync::Mutex` is enough here — lock durations are brief
/// (in-memory edits plus an occasional small JSON write), so there is no
/// need for async-aware locking.
pub struct AppState {
    pub data: Arc<Mutex<AppData>>,
    pub scheduler_state: Arc<Mutex<SchedulerState>>,
}
