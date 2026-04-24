pub mod pause_menu;
pub mod typing_path;

use egui_dock::DockState;

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Emulator,
    CpuViewer,
    PpuViewer,
}

pub fn create_initial_dock_state() -> DockState<Tab> {
    DockState::new(vec![Tab::Emulator])
}