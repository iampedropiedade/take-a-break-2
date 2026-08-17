use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Runtime scheduling state, persisted separately from user-editable
/// `AppData` so a postpone/skip survives an app restart.
///
/// Times are tracked as naive (timezone-less) wall-clock values, matching
/// what the OS reports as local time — this sidesteps DST ambiguity/gap
/// handling entirely, since nothing here ever needs a UTC offset.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulerState {
    /// The date a break last fired on, keyed by break id. Used to make
    /// firing idempotent within a given day regardless of tick timing.
    pub last_fired: HashMap<Uuid, NaiveDate>,
    /// A break the user postponed, and the local wall-clock time it should
    /// fire at.
    pub postponed: HashMap<Uuid, NaiveDateTime>,
    /// A break the user cancelled for a specific day; only that day's
    /// occurrence is skipped, future days are unaffected.
    pub skipped_today: HashMap<Uuid, NaiveDate>,
}
