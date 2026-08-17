use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value")]
pub enum BreakType {
    Hydration,
    Stretch,
    Eat,
    Custom(String),
}

impl BreakType {
    /// A short label used wherever the break needs a display name — since a
    /// break's type already says what it is, there's no separate name field.
    pub fn label(&self) -> String {
        match self {
            BreakType::Hydration => "Hydration".to_string(),
            BreakType::Stretch => "Stretch".to_string(),
            BreakType::Eat => "Eat".to_string(),
            BreakType::Custom(label) => label.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    Notification,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Break {
    pub id: Uuid,
    pub break_type: BreakType,
    pub start_time: NaiveTime,
    pub duration_minutes: u32,
    pub days: HashSet<Weekday>,
    /// `None` means inherit `Settings::default_display_mode`.
    pub display_mode: Option<DisplayMode>,
    /// Filename only, resolved against `$APPDATA/images/`.
    pub image_filename: Option<String>,
    pub message: String,
    pub enabled: bool,
}
