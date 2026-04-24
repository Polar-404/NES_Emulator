use std::path::PathBuf;

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