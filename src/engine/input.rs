use crate::memory::joypads::JoyPadButtons;
use macroquad::prelude::*;

use crate::engine::instance::EmulatorInstance;

/// uses the default values to map most of the inputs of the emulator instance
pub fn map_default_inputs(emulator_instance: &mut EmulatorInstance) {
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
    if is_key_pressed(KeyCode::Period) {
        emulator_instance.cpu.bus.ppu.color_palette.cycle_palettes();
    }

    if is_key_pressed(KeyCode::F1) {
        //TODO ver um jeito de ajustar para escalar a janela para o tamanho certo
        // ou simplesmente colocar uma tela preta no lugar das infos quando eu esconder elas
        emulator_instance.show_debug_info = !emulator_instance.show_debug_info;
    }
}

///updates a certain **"Gamepad Button"**, it can map multiple diferent keyboard buttons to the same gamepad button
pub fn update_gamepad_button(emulator_instance: &mut EmulatorInstance, joypad_btn: JoyPadButtons, target_keys: &[KeyCode]) {
    let keys = target_keys.iter().any(|&key| is_key_down(key));

    emulator_instance.cpu.bus.joypad_1.set_button(joypad_btn, keys);
}