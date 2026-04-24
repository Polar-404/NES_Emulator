use serde::{Serialize, Deserialize};
use crate::ppu::palettes::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmulatorConfig {
    pub palette: PaletteTheme,
    pub multiply_resolution: i32,
    pub allow_opposite_directions: bool
}
impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            palette: PaletteTheme::DefaultNtsc, 
            multiply_resolution: 2,
            allow_opposite_directions: true
        }
    }
}