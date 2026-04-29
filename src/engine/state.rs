use std::path::PathBuf;

#[cfg(feature = "debug_log")]
use crate::cpu::cpu::CPU;
use crate::{
    engine::instance::EmulatorInstance
};

pub enum EmulatorState {
    Menu,

    TypingPath,

    Loading { game_path: PathBuf },

    Running { 
        emulator_instance: EmulatorInstance, 
        #[cfg(feature = "debug_log")]
        logger: Box<dyn FnMut(&mut CPU)>,
    }
}