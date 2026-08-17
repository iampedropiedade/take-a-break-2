use serde::{Deserialize, Serialize};

use super::break_item::DisplayMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub paused: bool,
    pub default_display_mode: DisplayMode,
    pub cancel_on_call: bool,
    pub autostart: bool,
    /// If set, a fullscreen break shows on every connected monitor instead
    /// of just the primary one.
    #[serde(default)]
    pub show_on_all_screens: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            paused: false,
            default_display_mode: DisplayMode::Fullscreen,
            cancel_on_call: false,
            autostart: false,
            show_on_all_screens: false,
        }
    }
}
