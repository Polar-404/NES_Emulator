use std::path::{Path, PathBuf};

use arboard::Clipboard;
use macroquad::prelude::*;

use crate::engine::config::MULTIPLY_RESOLUTION;

use crate::engine::input::InputManager;
use crate::engine::state::EmulatorState;
use crate::{
    cpu::cpu::CPU,
    engine::stats::PerfomanceStats,
    memory,
    ui::pause_menu::*,
    apu::audio::AudioOutput,
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

    pub skin: macroquad::ui::Skin,
    pub smoothed_fps: f32,

} impl EmulatorInstance {
    pub fn new(game_path: PathBuf) -> Result<EmulatorInstance, Box<dyn std::error::Error>> {
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

            skin: create_customized_skin(MULTIPLY_RESOLUTION as f32),

            smoothed_fps: 60.0,
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

    pub fn push_formated_stats(&mut self) {
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

    pub async fn rungame(
        &mut self, 
        audio: &mut Option<(AudioOutput, u32)>, 
        #[cfg(feature = "debug_log")]
        logger: &mut Box<dyn FnMut(&mut CPU)>) {

        //let mut log_file = std::fs::File::create("nestest_output.log").unwrap();
        //let mut cycles_since_sample: f64 = 0.0;

        //let mut sample_count = 0;
        //let mut sample_sum = 0.0;

        self.cpu.bus.ppu.frame_complete = false;

        while !self.cpu.bus.ppu.frame_complete {
            //self.cpu.log_state(&mut log_file);

            #[cfg(feature = "debug_log")]
            let (_, cycles) = self.cpu.step_with_callback(Some(|cpu: &mut CPU| logger(cpu)));

            #[cfg(not(feature = "debug_log"))]
            let (_, cycles) = self.cpu.step();

            if let Some(ref mut audio) = audio {
                self.cpu.bus.sync_audio(cycles, audio);
            }
        }

        //at the end of each step, takes the frame buffer and updates the ppu texture with it, therefore rendering a new frame
        self.image.bytes.copy_from_slice(&self.cpu.bus.ppu.frame_buffer);
        self.ppu_texture.update(&self.image);

        let (source_rect, draw_width, draw_height) = if self.hide_overscan {
            (
                Some(Rect::new(8.0, 8.0, 240.0, 224.0)),
                240.0,
                224.0
            )
        } else {
            (
                None,
                256.0,
                240.0
            )
        };

        draw_texture_ex(
            &self.ppu_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    draw_width * (2.0 * MULTIPLY_RESOLUTION as f32), 
                    draw_height * (2.0 * MULTIPLY_RESOLUTION as f32)
                )), 
                source: source_rect,
                ..Default::default()
            },
        );
    }
}

pub fn load_game(game_path: PathBuf) -> Result<EmulatorState, Box<dyn std::error::Error>> {
    draw_text("CARREGANDO...", 200.0, 200.0, 50.0, YELLOW);

    let emu_instance = EmulatorInstance::new(game_path)?;

    //let emu_instance = match EmulatorInstance::new(game_path.to_path_buf()) {
    //    Ok(emu) => emu,
    //    Err(err) => {
    //        eprint!("[ERROR] An error occoured while Loading the ROM: {}", err);
    //        state = EmulatorState::Menu;
    //        path_buffer.clear();
    //        continue;
    //    }
    //};

    #[cfg(feature = "debug_log")]
    let logger = Box::new(ppu_debug::log_ppu(
        Some(".log/ppu_log.txt"),
        100_000,
        {
            let mut loop_counter = 0u32;
            move |cpu: &mut CPU| {
                if cpu.bus.ppu.frame_complete {
                    loop_counter += 1;
                }
                
                cpu.cycles >= 100_000_000 || loop_counter >= 10_000
            }
        }
    ));

    let audio = AudioOutput::new(44100);

    //print_program(&emu_instance);
    println!("tipo de mirroring: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
    
    Ok(EmulatorState::Running { 
        emulator_instance: emu_instance, 
        audio,
        input_manager: InputManager::new(),
        #[cfg(feature = "debug_log")]
        logger
    })
}