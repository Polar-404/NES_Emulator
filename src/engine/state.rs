use std::path::PathBuf;

use crate::{
    engine::instance::*,
    AudioOutput
};

pub enum EmulatorState {
    Menu,

    TypingPath,

    Loading { game_path: PathBuf },

    Running { 
        emulator_instance: EmulatorInstance, 
        audio: Option<(AudioOutput, u32)>, 
        #[cfg(feature = "debug_log")]
        logger: Box<dyn FnMut(&mut CPU)>
    }
}