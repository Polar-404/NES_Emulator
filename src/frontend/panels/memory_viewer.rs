use crate::{engine::instance::EmulatorInstance};

pub struct MemViewer {
    memory: Box<[u8]>
}
impl MemViewer {
    pub fn new() -> Self {
        Self {
            memory: vec![0u8; 0x07FF].into_boxed_slice()
        }
    }
    pub fn get_mem(&mut self, emu: &EmulatorInstance) {
        for i in 0..=0x7FF {
            self.memory[i] = emu.cpu.bus.peek(i as u16)
        }
    }
    pub fn render_memory_viewer(ui: &mut egui::Ui, emu: &EmulatorInstance, start_addr: u16, end_addr: u16) {
        ui.group(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("RAM: ($0000 up to $0x7FF)")
                        .size(12.0)
                        .strong()
                );
            });

            ui.separator();

            egui::ScrollArea::vertical().show_rows(ui, 16.0, ((end_addr - start_addr) / 16) as usize, |ui, row_range| {
                for row in row_range {
                    let addr = start_addr + (row as u16 * 16);
                    let mut row_text = format!("{:04X}:", addr);
                    
                    for i in 0..16 {
                        let val = emu.cpu.bus.peek(addr + i);
                        row_text.push_str(&format!(" {:02X}", val));
                    }
                    ui.label(egui::RichText::new(row_text).monospace());
                }
            });
        });
    }
}
