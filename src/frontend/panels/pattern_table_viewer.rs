use egui::{Color32, ColorImage, TextureHandle, TextureOptions};
use crate::engine::instance::EmulatorInstance;

pub struct PatternTableViewer {
    texture: Option<TextureHandle>,
}

impl PatternTableViewer {
    pub fn new() -> Self {
        Self { texture: None }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, emu: &EmulatorInstance) {
        
        let image = self.generate_pattern_image(emu);

        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture("pattern_table", image.clone(), TextureOptions::NEAREST)
        });

        texture.set(image, TextureOptions::NEAREST);

        let size = egui::vec2(256.0, 128.0);
        let image = egui::Image::new(egui::load::SizedTexture::new(texture.id(), size));
        ui.add(image);
    }

    fn generate_pattern_image(&self, emu: &EmulatorInstance) -> ColorImage {
        let mut image = ColorImage::new([256, 128], Color32::BLACK);

        for side in 0..2 {
            for tile_y in 0..16 {
                for tile_x in 0..16 {
                    let offset = tile_y * 256 + tile_x * 16;
                    
                    for row in 0..8 {

                        let addr = (side * 0x1000) + offset + row;
                        let tile_lsb = emu.cpu.bus.ppu.ppubus.peek(addr);
                        let tile_msb = emu.cpu.bus.ppu.ppubus.peek(addr + 8);

                        for col in 0..8 {
                            let pixel_x = (side * 128) + (tile_x * 8) + col;
                            let pixel_y = (tile_y * 8) + row;

                            let bit_lsb = (tile_lsb >> (7 - col)) & 1;
                            let bit_msb = (tile_msb >> (7 - col)) & 1;
                            let pixel_val = (bit_msb << 1) | bit_lsb;

                            let color = match pixel_val {
                                0 => Color32::BLACK,
                                1 => Color32::DARK_GRAY,
                                2 => Color32::LIGHT_GRAY,
                                3 => Color32::WHITE,
                                _ => unreachable!(),
                            };

                            image[(pixel_x as usize, pixel_y as usize)] = color;
                        }
                    }
                }
            }
        }
        image
    }
}