pub mod pause_menu;
pub mod typing_path;
pub mod cpu_viewer;
pub mod open_rom;
pub mod pattern_table_viewer;

use crate::frontend::dock_state::Tab;

use egui_dock::{DockState, NodeIndex};

pub fn create_initial_dock_state() -> DockState<Tab> {
    let mut state = DockState::new(vec![Tab::Emulator]);
    
    let [main, right] = state.main_surface_mut().split_right(
        NodeIndex::root(), 
        0.7,
        vec![Tab::CpuViewer, Tab::PpuViewer, Tab::Settings],
    );
    
    state.main_surface_mut().split_below(
        right,
        0.5,
        vec![Tab::MemoryEditor],
    );
    
    state
}