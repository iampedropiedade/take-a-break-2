use serde::{Deserialize, Serialize};

use super::{break_item::Break, settings::Settings};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub schema_version: u32,
    pub settings: Settings,
    pub breaks: Vec<Break>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            settings: Settings::default(),
            breaks: Vec::new(),
        }
    }
}
