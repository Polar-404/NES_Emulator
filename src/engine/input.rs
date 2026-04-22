use crate::memory::joypads::JoyPadButtons;
use macroquad::prelude::*;

use crate::engine::instance::EmulatorInstance;


fn update_join(emulator_instance: &mut EmulatorInstance) {
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::A, is_key_down(KeyCode::J) || is_key_down(KeyCode::Z) 
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::B, is_key_down(KeyCode::K) || is_key_down(KeyCode::X)
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::SELECT, is_key_down(KeyCode::N) || is_key_down(KeyCode::C)
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::START, is_key_down(KeyCode::M) || is_key_down(KeyCode::V)
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::UP, is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::DOWN, is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::LEFT, is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)
    );
    emulator_instance.cpu.bus.joypad_1.set_button(
        JoyPadButtons::RIGHT, is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)
    );

    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
        emulator_instance.cpu.bus.apu.volume = (emulator_instance.cpu.bus.apu.volume + 0.1).min(2.0);
    }
    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        emulator_instance.cpu.bus.apu.volume = (emulator_instance.cpu.bus.apu.volume - 0.1).max(0.0);
    }
    
    if is_key_pressed(KeyCode::Escape) {
        emulator_instance.is_paused = !emulator_instance.is_paused;
    }
}