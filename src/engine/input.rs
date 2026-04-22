use crate::memory::joypads::JoyPadButtons;
use macroquad::{prelude::*, ui::widgets::Button};

use crate::engine::instance::EmulatorInstance;

///JoyPad
pub enum InputAction {
    Joypad(JoyPadButtons),

    VolumeUp,
    VolumeDown,
    Pause,
    CyclePalette,
    ToggleDebug,
}

pub struct InputManager {
    pub joypad: Vec<(JoyPadButtons, Vec<KeyCode>)>,

    pub vol_up:         Vec<KeyCode>,
    pub vol_down:       Vec<KeyCode>,
    pub pause:          Vec<KeyCode>,
    pub cycle_palette:  Vec<KeyCode>,
    pub debug_info:     Vec<KeyCode>,
}

impl InputManager {
    pub fn new() -> Self {
        InputManager { 
            joypad: vec![
                (JoyPadButtons::A, vec![KeyCode::J, KeyCode::Z]),
                (JoyPadButtons::B, vec![KeyCode::K, KeyCode::X]),
                (JoyPadButtons::SELECT, vec![KeyCode::N, KeyCode::C]),
                (JoyPadButtons::START,  vec![KeyCode::M, KeyCode::V]),

                (JoyPadButtons::UP,     vec![KeyCode::W, KeyCode::Up]),
                (JoyPadButtons::DOWN,   vec![KeyCode::S, KeyCode::Down]),
                (JoyPadButtons::LEFT,   vec![KeyCode::A, KeyCode::Left]),
                (JoyPadButtons::RIGHT,  vec![KeyCode::D, KeyCode::Right]),
            ], 

            vol_up:         vec![KeyCode::Equal, KeyCode::KpAdd],
            vol_down:       vec![KeyCode::Minus, KeyCode::KpSubtract],
            pause:          vec![KeyCode::Escape],
            cycle_palette:  vec![KeyCode::Period],
            debug_info:     vec![KeyCode::F1],
        }
    }
    pub fn change_input_keys(&mut self, action: InputAction, new_key: KeyCode) {
        for (_, buttons) in &mut self.joypad {
            buttons.retain(|&k| k != new_key);
        }

        self.vol_up
            .retain(|&k| k != new_key);
        self.vol_down
            .retain(|&k| k != new_key);
        self.pause
            .retain(|&k| k != new_key);
        self.cycle_palette
            .retain(|&k| k != new_key);
        self.debug_info
            .retain(|&k| k != new_key);

        match action {
            InputAction::Joypad(btn) => {
                if let Some((_, keys)) = self.joypad.iter_mut().find(|(b, _)| *b == btn) {                 
                    keys.push(new_key);
                }
            }
            InputAction::VolumeUp =>        self.vol_up.push(new_key),
            InputAction::VolumeDown =>      self.vol_down.push(new_key),
            InputAction::Pause =>           self.pause.push(new_key),
            InputAction::CyclePalette =>    self.cycle_palette.push(new_key),
            InputAction::ToggleDebug =>     self.debug_info.push(new_key),
        }
    }

    pub fn tick(&self, emulator_instance: &mut EmulatorInstance) {
        for (button, keys) in &self.joypad {
            let is_pressed = keys.iter().any(|&k| is_key_down(k));
            emulator_instance.cpu.bus.joypad_1.set_button(*button, is_pressed);
        }
        if self.is_any_pressed(&self.vol_up) {
            emulator_instance.cpu.bus.apu.volume = (emulator_instance.cpu.bus.apu.volume + 0.1).min(2.0);
        }
        if self.is_any_pressed(&self.vol_down) {
            emulator_instance.cpu.bus.apu.volume = (emulator_instance.cpu.bus.apu.volume - 0.1).max(0.0);
        }
        if self.is_any_pressed(&self.pause) {
            emulator_instance.is_paused = !emulator_instance.is_paused;
        }
        if self.is_any_pressed(&self.cycle_palette) {
            emulator_instance.cpu.bus.ppu.color_palette.cycle_palettes();
        }
        if self.is_any_pressed(&self.debug_info) {
            emulator_instance.show_debug_info = !emulator_instance.show_debug_info;
        }
    }
    fn is_any_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&k| is_key_pressed(k)) 
    }
}

