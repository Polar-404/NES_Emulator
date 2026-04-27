use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use crate::engine::instance::EmulatorInstance;

pub struct PaletteViewer {
    texture: Option<TextureHandle>,
}

impl PaletteViewer {
    pub fn new() -> Self {
        Self { texture: None }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, emu: &EmulatorInstance) {
        let image = self.generate_palette_image(emu);

        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture("ppu_palettes", image.clone(), TextureOptions::NEAREST)
        });

        texture.set(image, TextureOptions::NEAREST);

        ui.group(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("PPU Palettes ($3F00 - $3F1F)")
                        .size(12.0)
                        .strong()
                );
            });

            ui.separator();

            let size = egui::vec2(ui.available_width(), ui.available_width() / 4.0);

            let image = egui::Image::new(egui::load::SizedTexture::new(texture.id(), size));
            ui.add(image);
        });
    }

    fn generate_palette_image(&self, emu: &EmulatorInstance) -> ColorImage {
        let mut image = ColorImage::new([256, 64], Color32::BLACK);
        let rgb = emu.cpu.bus.ppu.color_palette.get_collors();

        for is_sprite in 0..2 {
            for palette_idx in 0..4 {
                for color_idx in 0..4 {
                    let addr = 0x3F00 + (is_sprite * 16) + (palette_idx * 4) + color_idx;
                    let nes_color_idx = emu.cpu.bus.ppu.ppubus.peek(addr) & 0x3F;
                    
                    let color = Color32::from_rgb(
                        rgb[nes_color_idx as usize].r,
                        rgb[nes_color_idx as usize].g,
                        rgb[nes_color_idx as usize].b,
                    );

                    let base_x = (palette_idx * 4 + color_idx) * 16;
                    let base_y = is_sprite * 16;

                    for y in 0..16 {
                        for x in 0..16 {
                            if y > 13 || (color_idx == 3 && x > 13) {
                                image[((base_x + x) as usize, (base_y + y) as usize)] = Color32::BLACK;
                            } else {
                                image[((base_x + x) as usize, (base_y + y) as usize)] = color;
                            }
                        }
                    }
                }
            }
        }
        image
    }
}