use egui::TextureId;
use egui_dock::{DockArea, DockState, Style};
use egui_glow::EguiGlow;
use glow::HasContext;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop},
    window::{Window, WindowId},
};

use crate::{
    engine::instance::EmulatorInstance, 
    frontend::{
        dock_state::{NesTabViewer, Tab}, glstate::GLState, nes_texture::NesTexture, panels::create_initial_dock_state
    }
};

use std::{sync::Arc};

pub struct App {
    window: Option<Arc<Window>>,
    gl_state: Option<GLState>,        // glutin + glow
    egui_glow: Option<EguiGlow>,      // egui rendering
    dock_state: DockState<Tab>,

    nes: Option<EmulatorInstance>,
    nes_texture: Option<NesTexture>,
}
impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gl_state: None,
            egui_glow: None,
            dock_state: create_initial_dock_state(),

            nes: None,
            nes_texture: None,
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
        if let Some(egui_glow) = &mut self.egui_glow {
            let response = egui_glow.on_window_event(self.window.as_ref().unwrap(), &event);
            if response.consumed { return; }
        }
        
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

                if self.nes_texture.is_none() && !self.nes.is_none() {
                    self.nes_texture = Some(NesTexture::new(gl, egui_glow));
                }

                if let Some(emu) = &mut self.nes {
                    emu.run_frame(&mut None);
                    let texture = self.nes_texture.as_ref().unwrap();
                    texture.update(gl, emu.frame_buffer());
                }

                let mut open_rom_requested = false;   

                let dock = &mut self.dock_state;
                let nes_ref = self.nes.as_ref();
                let texture = &self.nes_texture; 

                let texture_opt = self.nes_texture.as_ref().map(|nt| nt.egui_texture_id);

                let repaint_after = egui_glow.run(window, |ctx| { 
                    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            ui.menu_button("File", |ui| {
                                if ui.button("Open ROM...").clicked() {
                                    open_rom_requested = true;
                                    ui.close_menu();
                                }
                                if ui.button("Exit").clicked() { std::process::exit(0); }
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
                    
                    DockArea::new(dock)
                    .style(Style::from_egui(ctx.style().as_ref()))
                    .show(ctx, &mut NesTabViewer {
                        nes_texture: texture_opt,
                        emulator: nes_ref,
                    });
                });

                if open_rom_requested {
                    if let Some(path) = crate::frontend::panels::open_rom::open_rom_dialog() {
                        match crate::engine::instance::EmulatorInstance::new(path) {
                            Ok(emu) => {
                                self.nes = Some(emu);
                            }
                            Err(e) => {
                                eprintln!("Falha ao carregar a ROM: {}", e);
                            }
                        }
                    }
                }

                egui_glow.paint(window);
                gl_state.swap_buffers();
                window.request_redraw();

            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                
            }
            _ => { }
        }
    }
}