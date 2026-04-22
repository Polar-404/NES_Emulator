use macroquad::prelude::*;
use arboard::Clipboard;

use crate::{
    apu::audio::AudioOutput, engine::{
        config::*, state::*
    }, 
    ui::{
        pause_menu::*, typing_path::handle_type_pathing
    }
};

#[cfg(feature = "debug_log")]
use crate::debug::{cpu_debug, ppu_debug};


mod cpu;
mod memory;
mod ppu;
mod apu;
mod ui;
mod debug;
mod engine;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

fn window_conf() -> Conf {
    Conf {
        window_title: "NES Emulator".to_owned(),

        window_width: 720 * MULTIPLY_RESOLUTION,
        window_height: 480 * MULTIPLY_RESOLUTION,

        sample_count: 0,

        high_dpi: true,
        fullscreen: false,

        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::X11Only,
            ..Default::default()
        },
        ..Default::default()

    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let mut clipboard = Clipboard::new().ok();
    let mut state = EmulatorState::Menu;
    let mut path_buffer = String::new();

    let frame_time: std::time::Duration = std::time::Duration::from_secs_f64(1.0/60.0);
    let mut frame_deadline: std::time::Instant = std::time::Instant::now();

    loop {
        match &mut state {
            // TODO implement more 'Menu' funcionalities
            EmulatorState::Menu => {
                clear_background(DARKBLUE);

                draw_text("Press any key to continue", 20.0, 40.0, 30.0, WHITE);
                if get_last_key_pressed().is_some() {
                    state = EmulatorState::TypingPath;
                    path_buffer.clear();

                    // drains the charactere queue so it doesnt enter the path_buffer
                    while get_char_pressed().is_some() {}
                }
            }
            EmulatorState::TypingPath => {
                state = handle_type_pathing(&mut path_buffer, &mut clipboard)
            }

            EmulatorState::Loading { game_path } => {
                match engine::instance::load_game(game_path.to_path_buf()) {
                    Ok(running_state) => state = running_state,
                    Err(err) => {
                        eprint!("[ERROR] An error occoured while Loading the ROM: {}", err);
                        state = EmulatorState::Menu;
                        path_buffer.clear();
                        continue;
                    }
                }
            }

            EmulatorState::Running { 
                ref mut emulator_instance, 
                ref mut audio, 
                ref mut input_manager,
                #[cfg(feature = "debug_log")]
                ref mut logger 
            } => {

                // gets user key inputs
                input_manager.tick(emulator_instance);
                
                clear_background(BLACK);

                if !emulator_instance.is_paused {
                    emulator_instance.rungame(audio, #[cfg(feature = "debug_log")] logger).await;

                    emulator_instance.smoothed_fps = (emulator_instance.smoothed_fps * 0.9) + (get_fps() as f32 * 0.1);
                    draw_text(&get_fps().to_string(), 10.0, 20.0, 30.0, WHITE);
                    emulator_instance.show_debug_info();
                } else {
                    if render_pause_menu(emulator_instance) {
                        state = EmulatorState::Menu
                    }
                }
            }
        }

        next_frame().await;

        let now = std::time::Instant::now();
        if now < frame_deadline {
            std::thread::sleep(frame_deadline - now);
        }
        frame_deadline += frame_time

    }
}