use std::path::PathBuf;

use macroquad::prelude::*;
use arboard::Clipboard;

use crate::{
    apu::audio::AudioOutput,
    ui::pause_menu::*,

    engine::{
        config::*,
        instance::*,
        input::*,
        state::*,
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
            }

            EmulatorState::Loading { game_path } => {
                draw_text("CARREGANDO...", 200.0, 200.0, 50.0, YELLOW);

                let emu_instance = match EmulatorInstance::new(game_path.to_path_buf()) {
                    Ok(emu) => emu,
                    Err(err) => {
                        eprint!("[ERROR] An error occoured while Loading the ROM: {}", err);
                        state = EmulatorState::Menu;
                        path_buffer.clear();
                        continue;
                    }
                };

                #[cfg(feature = "debug_log")]
                let logger = Box::new(ppu_debug::log_ppu(
                    Some(".log/ppu_log.txt"),
                    100_000,
                    {
                        let mut loop_counter = 0u32;
                        move |cpu: &mut CPU| {
                            if cpu.bus.ppu.frame_complete {
                                loop_counter += 1;
                            }
                            
                            cpu.cycles >= 100_000_000 || loop_counter >= 10_000
                        }
                    }
                ));

                let audio = AudioOutput::new(44100);

                //print_program(&emu_instance);
                println!("tipo de mirroring: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
                
                state = EmulatorState::Running { 
                    emulator_instance: emu_instance, 
                    audio,
                    #[cfg(feature = "debug_log")]
                    logger
                };
            }

            EmulatorState::Running { 
                ref mut emulator_instance, 
                ref mut audio, 
                #[cfg(feature = "debug_log")]
                ref mut logger 
            } => {

                map_default_inputs(emulator_instance);
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