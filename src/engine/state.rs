use std::path::PathBuf;

use crate::{
    apu::audio::AudioOutput, engine::{input::InputManager, instance::*}
};

pub enum EmulatorState {
    Menu,

    TypingPath,

    Loading { game_path: PathBuf },

    Running { 
        emulator_instance: EmulatorInstance, 
        audio: Option<(AudioOutput, u32)>, 
        input_manager: InputManager,
        #[cfg(feature = "debug_log")]
        logger: Box<dyn FnMut(&mut CPU)>,
    }
}