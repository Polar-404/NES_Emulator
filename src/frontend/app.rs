use egui_dock::DockState;
use egui_glow::EguiGlow;
use glow::HasContext;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    frontend::panels::{Tab, create_initial_dock_state},
    engine::instance::EmulatorInstance,
    engine::state::EmulatorState, 
    frontend::glstate::GLState,
};

use std::sync::Arc;

pub struct App {
    window: Option<Arc<Window>>,
    gl_state: Option<GLState>,        // glutin + glow
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
            .unwrap()
        );
        let gl_state = GLState::new(event_loop, &window);
        let egui_glow = EguiGlow::new(event_loop, Arc::clone(&gl_state.gl), None, None, true);
        
        
        self.window = Some(window);
        self.gl_state = Some(gl_state);
        self.egui_glow = Some(egui_glow);
    }
    fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
        match event {
            WindowEvent::RedrawRequested => {
                let gl_state = self.gl_state.as_ref().unwrap();
                let egui_glow = self.egui_glow.as_mut().unwrap();
                let window = self.window.as_ref().unwrap();
                let gl = &gl_state.gl;

                unsafe {
                    gl.clear_color(0.1, 0.1, 0.1, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                let repaint_after = egui_glow.run(window, |ctx| { 
                    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            ui.menu_button("File", |ui| {
                                if ui.button("Open ROM...").clicked() {
                                    todo!()
                                }
                                if ui.button("Exit").clicked() {
                                    std::process::exit(0);
                                }
                            });
                            ui.menu_button("NES", |ui| {
                                if ui.button("Reset").clicked() { /* reset */ }
                                if ui.button("Pause").clicked() { /* pause */ }
                            });
                            ui.menu_button("Debug", |ui| {
                                if ui.button("CPU Viewer").clicked() { /* toggle */ }
                                if ui.button("PPU Viewer").clicked() { /* toggle */ }
                            });
                        });
                    });
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("NES Framebuffer vai aqui");
                    });
                });

                egui_glow.paint(window);
                gl_state.swap_buffers();
                window.request_redraw();
                
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => { }
        }
    }
}
