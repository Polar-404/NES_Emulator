use egui_dock::{TabViewer};
use crate::engine::instance::EmulatorInstance;

use crate::frontend::panels::cpu_viewer::render_cpu_viewer;

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Emulator,   
    CpuViewer,
    PpuViewer,  
    MemoryEditor,
    ApuWaveform,
}

pub struct NesTabViewer<'a> {
    pub nes_texture: Option<egui::TextureId>,
    pub emulator: Option<&'a EmulatorInstance>,
}

impl TabViewer for NesTabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        match tab {
            Tab::Emulator    => "NES".into(),
            Tab::CpuViewer   => "CPU".into(),
            Tab::PpuViewer   => "PPU".into(),
            Tab::MemoryEditor => "Memory".into(),
            Tab::ApuWaveform => "APU".into(),
        }
    }
    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Emulator => {
                if let Some(tex_id) = self.nes_texture {
                    let available = ui.available_size();
                    
                    let scale = (available.x / 256.0).min(available.y / 240.0);
                    let size = egui::vec2(256.0 * scale, 240.0 * scale);
                    ui.centered_and_justified(|ui| {
                        ui.image(egui::load::SizedTexture::new(tex_id, size));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Waiting to start video system...");
                    });
                }
            }
            Tab::CpuViewer => {
                if let Some(emu) = self.emulator {
                    render_cpu_viewer(ui, emu);
                } else {
                    ui.label("No Game loaded, insert a ROM to view the CPU");
                }
                
            }
            Tab::PpuViewer => {
                // pattern tables etc
            }
            Tab::MemoryEditor => {
                // hex view
            }
            Tab::ApuWaveform => {
                // waveform plot
            }
        }
    }
}