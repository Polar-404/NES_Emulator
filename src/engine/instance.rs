use std::path::{Path, PathBuf};

use macroquad::prelude::*;

use crate::engine::config::EmulatorConfig;

use crate::engine::state::EmulatorState;
use crate::{
    cpu::cpu::CPU,
    engine::stats::PerfomanceStats,
    memory,
    apu::audio::AudioOutput,
};

pub struct EmulatorInstance {
    pub cpu: CPU,
    pub show_debug_info: bool,
    pub is_paused: bool,
    hide_overscan: bool,
    debug_frame_counter: u8,
    cached_debug_text: Vec<String>,
    stats: PerfomanceStats,
}

impl EmulatorInstance {
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
            cpu: cpu,
            show_debug_info: false, 
            is_paused: false,
            stats: PerfomanceStats::new(),
            hide_overscan: true,

            debug_frame_counter: 0,
            cached_debug_text: Vec::new(),
        })
    }

    pub async fn rungame(
        &mut self, 
        audio: &mut Option<(AudioOutput, u32)>, 
        #[cfg(feature = "debug_log")]
        logger: &mut Box<dyn FnMut(&mut CPU)>) 
        {

        self.cpu.bus.ppu.frame_complete = false;

        while !self.cpu.bus.ppu.frame_complete {

            #[cfg(feature = "debug_log")]
            let (_, cycles) = self.cpu.step_with_callback(Some(|cpu: &mut CPU| logger(cpu)));

            #[cfg(not(feature = "debug_log"))]
            let (_, cycles) = self.cpu.step();

            if let Some(ref mut audio) = audio {
                self.cpu.bus.sync_audio(cycles, audio);
            }
        }
    }
}

pub fn load_game(game_path: PathBuf) -> Result<EmulatorState, Box<dyn std::error::Error>> {
    let emu_instance = EmulatorInstance::new(game_path)?;

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

    println!("Mirroring Type: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
    
    Ok(EmulatorState::Running { 
        emulator_instance: emu_instance, 
        audio,
        input_manager: InputManager::new(),
        #[cfg(feature = "debug_log")]
        logger
    })
}