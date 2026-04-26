use std::{collections::HashMap, path::PathBuf};

use serde::{Serialize, Deserialize};
use crate::ppu::palettes::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmulatorConfig {
    pub volume: f32,
    pub hide_overscan: bool,
    pub palette: PaletteTheme,
    pub multiply_resolution: i32,
    pub allow_opposite_directions: bool,
    pub custom_palettes: HashMap<String, Vec<NESColor>>,
}
impl EmulatorConfig {
    pub fn load() -> Self {
        std::fs::read_to_string("config.json")
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }
    fn get_config_path() -> PathBuf {
        let mut config_path = PathBuf::new();
        config_path.push("config");
        config_path.set_extension("json");
        config_path
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let config_path = Self::get_config_path();
            let _ = std::fs::write(config_path, content);
        }
    }
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            volume: 10.0,
            hide_overscan: true,
            palette: PaletteTheme::DefaultNtsc, 
            multiply_resolution: 2,
            allow_opposite_directions: true,
            custom_palettes: HashMap::new(),
        }
    }
}