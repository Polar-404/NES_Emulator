pub mod pause_menu;
pub mod typing_path;
pub mod cpu_viewer;
pub mod open_rom;
pub mod memory_viewer;
pub mod settings_panel;
pub mod app_terminal;

pub mod ppu_viewer;

use crate::frontend::dock_state::Tab;

use egui_dock::{DockState, NodeIndex};

pub fn create_initial_dock_state() -> DockState<Tab> {
    let mut state = DockState::new(vec![Tab::Emulator]);
    
    let [_main, right] = state.main_surface_mut().split_right(
        NodeIndex::root(), 
        0.55,
        vec![Tab::PpuViewer, Tab::CpuViewer],
    );
    
    let [_main2, down2] = state.main_surface_mut().split_below(
        right,
        0.6,
        vec![Tab::MemoryEditor],
    );

    let [_main3, down3] = state.main_surface_mut().split_right(
        down2,
        0.5,
        vec![Tab::Terminal],
    );

    let [_main2, _right4] = state.main_surface_mut().split_below(
        down3,
        0.5,
        vec![Tab::Settings],
    );
    
    state
}