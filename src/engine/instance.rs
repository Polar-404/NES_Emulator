use std::path::{Path, PathBuf};

use macroquad::prelude::*;
use crate::engine::config::MULTIPLY_RESOLUTION;

use crate::{
    cpu::cpu::CPU,
    engine::stats::PerfomanceStats,
    memory,
};

pub struct EmulatorInstance {
    //emulator itself
    pub cpu: CPU, 

    pub image: Image, 
    pub ppu_texture: Texture2D, 

    pub show_debug_info: bool, 
    pub is_paused: bool,

    debug_frame_counter: u8,
    cached_debug_text: Vec<String>,

    stats: PerfomanceStats,
    hide_overscan: bool,

} impl EmulatorInstance {
    fn new(game_path: PathBuf) -> Result<EmulatorInstance, Box<dyn std::error::Error>> {
        let mapper = memory::bus::load_rom_from_file(Path::new(game_path.as_path()))?;
        let mut cpu = CPU::new(mapper);
        
        cpu.reset_interrupt();
        //FORCING NESTEST
        //cpu.program_counter = 0xC000; 

        let image = Image::gen_image_color(256, 240, Color { r: 0.0 , g: 0.0, b: 0.0, a: 1.0 });
        let ppu_texture = Texture2D::from_image(&image);

        ppu_texture.set_filter(FilterMode::Nearest);

        clear_background(BLACK);

        Ok(EmulatorInstance { 
            //mapper: mapper,
            cpu: cpu, 
            image: image, 
            ppu_texture: ppu_texture, 
            show_debug_info: false, 
            is_paused: false,
            stats: PerfomanceStats::new(),
            hide_overscan: true,

            debug_frame_counter: 0,
            cached_debug_text: Vec::new(),
        })
    }
    pub fn show_debug_info(&mut self) {
        if self.show_debug_info {
            let pos_x: f32 = 520.0 * MULTIPLY_RESOLUTION as f32;
            let mut pos_y: f32 = 30.0;
            let line_height = 30.0;
            let font_size = 30.0;

            self.debug_frame_counter = (self.debug_frame_counter + 1) % 4;

            if self.debug_frame_counter == 0 {
                self.push_formated_stats();
            }

            for line in &self.cached_debug_text {
                draw_text(line, pos_x, pos_y, font_size, WHITE);
                pos_y += line_height;
            }

        }
    }

    fn push_formated_stats(&mut self) {
        self.stats.update_status();

        self.cached_debug_text.clear();

        // general info
        self.cached_debug_text.push(format!("Vol: {:.0}%", self.cpu.bus.apu.volume * 100.0));
        self.cached_debug_text.push(format!("CPU: {:.1}% | Thread: {:.1}%", self.stats.cpu_usage, self.stats.main_emu_thread));
        self.cached_debug_text.push(format!("RAM: {:.2} MB", self.stats.memory_usage_mb));

        // empty space
        self.cached_debug_text.push(String::new());

        // cpu info
        self.cached_debug_text.push(format!("STATUS: {}", CPU::format_cpu_status(self.cpu.status.bits())));
        self.cached_debug_text.push(format!("PC: {:#06x}", self.cpu.program_counter));
        self.cached_debug_text.push(format!("CYCLES: {:?}", self.cpu.cycles));
        self.cached_debug_text.push(format!("A: {:#04x} | X: {:#04x} | Y: {:#04x}", self.cpu.register_a, self.cpu.register_x, self.cpu.register_y));

        // empty space
        self.cached_debug_text.push(String::new());

        // ppu info
        self.cached_debug_text.push(String::from("PPU INFO:"));
        self.cached_debug_text.push(format!("PPUCTRL: {:#010b} ({:#04x})", self.cpu.bus.ppu.ctrl.bits(), self.cpu.bus.ppu.ctrl.bits()));
        self.cached_debug_text.push(format!("PPUMASK: {:#010b} ({:#04x})", self.cpu.bus.ppu.mask, self.cpu.bus.ppu.mask));
        self.cached_debug_text.push(format!("PPUSTATUS: {:#010b} ({:#04x})", self.cpu.bus.ppu.status.bits(), self.cpu.bus.ppu.status.bits()));
        self.cached_debug_text.push(format!("VRAM: {:#06x} | T-VRAM: {:#06x}", self.cpu.bus.ppu.v.addr, self.cpu.bus.ppu.t.addr));
        self.cached_debug_text.push(format!("PPU Cycle: {} | Scanline: {}", self.cpu.bus.ppu.cycle, self.cpu.bus.ppu.scanline));
        self.cached_debug_text.push(format!("Frame Complete: {:?}", self.cpu.bus.ppu.frame_complete));
    }
}