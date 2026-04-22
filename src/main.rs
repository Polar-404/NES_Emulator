use std::path::{Path, PathBuf};

use macroquad::prelude::*;
use arboard::Clipboard;
use sysinfo::{System, Pid};

use crate::{
    cpu::cpu::CPU, 
    apu::audio::AudioOutput,
    ui::menu::*,
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

enum EmulatorState {
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
    let skin: macroquad::ui::Skin = create_customized_skin(MULTIPLY_RESOLUTION as f32);

    let mut smoothed_fps = 60.0;

    async fn rungame(
        emulator: &mut EmulatorInstance, 
        audio: &mut Option<(AudioOutput, u32)>, 
        #[cfg(feature = "debug_log")]
        logger: &mut Box<dyn FnMut(&mut CPU)>) {

        //let mut log_file = std::fs::File::create("nestest_output.log").unwrap();
        //let mut cycles_since_sample: f64 = 0.0;

        //let mut sample_count = 0;
        //let mut sample_sum = 0.0;

        emulator.cpu.bus.ppu.frame_complete = false;

        while !emulator.cpu.bus.ppu.frame_complete {
            //emulator.cpu.log_state(&mut log_file);

            #[cfg(feature = "debug_log")]
            let (_, cycles) = emulator.cpu.step_with_callback(Some(|cpu: &mut CPU| logger(cpu)));

            #[cfg(not(feature = "debug_log"))]
            let (_, cycles) = emulator.cpu.step();

            if let Some(ref mut audio) = audio {
                emulator.cpu.bus.sync_audio(cycles, audio);
            }
        }

        //at the end of each step, takes the frame buffer and updates the ppu texture with it, therefore rendering a new frame
        emulator.image.bytes.copy_from_slice(&emulator.cpu.bus.ppu.frame_buffer);
        emulator.ppu_texture.update(&emulator.image);

        let (source_rect, draw_width, draw_height) = if emulator.hide_overscan {
            (
                Some(Rect::new(8.0, 8.0, 240.0, 224.0)),
                240.0,
                224.0
            )
        } else {
            (
                None,
                256.0,
                240.0
            )
        };

        draw_texture_ex(
            &emulator.ppu_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    draw_width * (2.0 * MULTIPLY_RESOLUTION as f32), 
                    draw_height * (2.0 * MULTIPLY_RESOLUTION as f32)
                )), 
                source: source_rect,
                ..Default::default()
            },
        );
        
        if is_key_pressed(KeyCode::F1) {
            //TODO ver um jeito de ajustar para escalar a janela para o tamanho certo
            // ou simplesmente colocar uma tela preta no lugar das infos quando eu esconder elas
            emulator.show_debug_info = !emulator.show_debug_info;
        }
        if is_key_pressed(KeyCode::Period) {
            emulator.cpu.bus.ppu.color_palette.cycle_palettes();
        }

    }

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

                clear_background(BLACK);

                if !emulator_instance.is_paused {
                    rungame(emulator_instance, audio, #[cfg(feature = "debug_log")] logger).await;

                    smoothed_fps = (smoothed_fps * 0.9) + (get_fps() as f32 * 0.1);
                    draw_text(&get_fps().to_string(), 10.0, 20.0, 30.0, WHITE);
                    emulator_instance.show_debug_info();
                } else {
                    if render_pause_menu(emulator_instance, Some(&skin)) {
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