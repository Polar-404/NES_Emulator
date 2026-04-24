use std::path::{Path, PathBuf};

use crate::engine::config::EmulatorConfig;

use crate::{
    cpu::cpu::CPU,
    engine::stats::PerfomanceStats,
    memory,
    apu::audio::AudioOutput,
};

pub struct EmulatorInstance {
    pub cpu: CPU,
    pub is_paused: bool,
    stats: PerfomanceStats,
}

impl EmulatorInstance {
    pub fn new(game_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mapper = memory::bus::load_rom_from_file(Path::new(&game_path))?;
        let mut cpu = CPU::new(mapper);
        cpu.reset_interrupt();

        Ok(Self {
            cpu,
            is_paused: false,
            stats: PerfomanceStats::new(),
        })
    }

    pub fn run_frame(&mut self, audio: &mut Option<(AudioOutput, u32)>) {
        self.cpu.bus.ppu.frame_complete = false;

        if self.is_paused { return; }

        while !self.cpu.bus.ppu.frame_complete {
            #[cfg(feature = "debug_log")]
            let (_, cycles) = self.cpu.step_with_callback(Some(|cpu: &mut CPU| logger(cpu)));

            #[cfg(not(feature = "debug_log"))]
            let (_, cycles) = self.cpu.step();

            if let Some(audio) = audio {
                self.cpu.bus.sync_audio(cycles, audio);
            }
        }
    }

    pub fn frame_buffer(&self) -> &[u8] {
        &self.cpu.bus.ppu.frame_buffer
    }
}


// pub fn load_game(game_path: PathBuf) -> Result<EmulatorState, Box<dyn std::error::Error>> {
//     let emu_instance = EmulatorInstance::new(game_path)?;
// 
//     #[cfg(feature = "debug_log")]
//     let logger = Box::new(ppu_debug::log_ppu(
//         Some(".log/ppu_log.txt"),
//         100_000,
//         {
//             let mut loop_counter = 0u32;
//             move |cpu: &mut CPU| {
//                 if cpu.bus.ppu.frame_complete {
//                     loop_counter += 1;
//                 }
//                 
//                 cpu.cycles >= 100_000_000 || loop_counter >= 10_000
//             }
//         }
//     ));
// 
//     let audio = AudioOutput::new(44100);
// 
//     println!("Mirroring Type: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
//     
//     Ok(EmulatorState::Running { 
//         emulator_instance: emu_instance, 
//         audio,
//         input_manager: InputManager::new(),
//         #[cfg(feature = "debug_log")]
//         logger
//     })
// }