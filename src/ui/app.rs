use egui_dock::DockState;
use egui_glow::EguiGlow;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::engine::{
    state::EmulatorState
};

use std::sync::Arc;

pub struct App {
    window: Option<Arc<Window>>,
    gl_state: Option<GlState>,        // glutin + glow
    egui_glow: Option<EguiGlow>,      // egui rendering
    dock_state: DockState<Tab>,
    emulator_state: EmulatorState,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gl_state: None,
            egui_glow: None,
            dock_state: create_initial_dock_state(),
            emulator_state: EmulatorState::Menu,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                .with_title("NES Emulator")
                .with_inner_size(
                    winit::dpi::LogicalSize::new(1280, 720)
                )
            )
        );
    }
    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => { }
        }
    }
}
