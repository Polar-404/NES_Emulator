use std::path::PathBuf;

use arboard::Clipboard;
use macroquad::prelude::*;

use crate::engine::state::EmulatorState;

pub fn handle_type_pathing(path_buffer: &mut String , clipboard: &mut Option<Clipboard>) -> EmulatorState {
    let mut state = EmulatorState::TypingPath;

    clear_background(DARKBLUE);

    draw_text("Insert the game file path (.nes file):", 20.0, 50.0, 40.0, BLACK);

    while let Some(c) = get_char_pressed() {
    // filtel control characters
        if !c.is_control() {
            path_buffer.push(c);
        }
    }

    if is_key_pressed(KeyCode::Backspace) {
        path_buffer.pop();
    }

    if is_key_pressed(KeyCode::Enter) && !path_buffer.is_empty() {
        state = EmulatorState::Loading { game_path: PathBuf::from(path_buffer.trim()) }
    }

    if is_key_pressed(KeyCode::Escape) {
        path_buffer.clear();
        state = EmulatorState::Menu;
    }
    if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::V) {
        if let Some(ref mut cb) = clipboard {
            if let Ok(text) = cb.get_text() {
                path_buffer.push_str(&text.replace("\n", "").replace("\r", ""));
            }
        }
    }

    let display_text = format!("{}_", path_buffer);
    draw_text(&display_text, 20.0, 100.0, 30.0, YELLOW);
    draw_text("Press ENTER to continue or ESC to cancel", 20.0, 150.0, 40.0, BLACK);

    state
}