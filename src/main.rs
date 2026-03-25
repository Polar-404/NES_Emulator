use std::path::{Path, PathBuf};

use macroquad::prelude::*;
use arboard::Clipboard;
use ringbuf::traits::{Observer as _, Producer as _};

use crate::{
    cpu::cpu::CPU, 
    memory::joypads::JoyPadButtons,
    apu::audio::AudioOutput,
};

mod cpu;
mod memory;
mod ppu;
mod apu;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;


const MULTIPLY_RESOLUTION: i32 = 2;

const DEFAULT_GAME_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/NES_GAMES/Mario/Super Mario Bros. (World).nes");
const DEFAULT_GAME_NAME: &str = "Super Mario Bros. (World)";

const CPU_FREQ: f64 = 1_789_773.0;

struct EmulatorInstance {
    cpu: CPU, 
    image: Image, 
    ppu_texture: Texture2D, 
    show_debug_info: bool, 
    is_paused: bool,
    cycles_since_sample: f64

} impl EmulatorInstance {
    fn new(game_path: PathBuf) -> EmulatorInstance {
        
        let mapper = memory::bus::load_rom_from_file(Path::new(game_path.as_path()));
        
        let mut cpu = CPU::new(mapper);
        
        cpu.reset_interrupt();

        //FORCING NESTEST
        //cpu.program_counter = 0xC000; 
        //cpu.status = CpuFlags::from_bits_truncate(0b100100);
        //cpu.stack_pointer = 0xFD; 
        //cpu.cycles = 7;

        let image = Image::gen_image_color(256, 240, Color { r: 0.0 , g: 0.0, b: 0.0, a: 1.0 });
        let ppu_texture = Texture2D::from_image(&image);

        ppu_texture.set_filter(FilterMode::Nearest);

        clear_background(BLACK);

        EmulatorInstance { 
            //mapper: mapper,
            cpu: cpu, 
            image: image, 
            ppu_texture: ppu_texture, 
            show_debug_info: false, 
            is_paused: false,
            cycles_since_sample: 0.0,
        }
    }

    #[inline]
    fn step(&mut self, audio: &mut Option<(AudioOutput, u32)>) {

        let opcode_cycles = self.cpu.step(|_| {}).1;

        if let Some(ref mut audio) = audio {
            self.cycles_since_sample += opcode_cycles as f64;

            let capacity = audio.0.producer.capacity().get() as f64;
            let len = audio.0.producer.occupied_len() as f64;
            let fullness = len / capacity; // goes from 0.0 (empty) up to 1.0 (full)

            let rate_adjustment = if fullness < 0.4 {
                0.98
            } else if fullness > 0.6 {
                1.02
            } else {
                1.0 
            };

            let cycles_per_sample = (CPU_FREQ / audio.1 as f64) * rate_adjustment;

            if self.cycles_since_sample >= cycles_per_sample {
                self.cycles_since_sample -= cycles_per_sample;
                let sample = self.cpu.bus.apu.get_sample();
                let _ = audio.0.producer.try_push(sample);
            }
        }
    }
    
}
enum EmulatorState {
    Menu,

    TypingPath,

    Loading { game_path: PathBuf },

    Running { emulator_instance: EmulatorInstance, audio: Option<(AudioOutput, u32)> }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "NES Emulator".to_owned(),

        window_width: 720 * MULTIPLY_RESOLUTION,
        window_height: 480 * MULTIPLY_RESOLUTION,

        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let mut clipboard = Clipboard::new().ok();
    let mut state = EmulatorState::Menu;
    let mut path_buffer = String::new();

    let frame_time = std::time::Duration::from_secs_f64(1.0/60.0);
    let mut frame_deadline = std::time::Instant::now();

    async fn rungame(emulator: &mut EmulatorInstance, audio: &mut Option<(AudioOutput, u32)>) {

        let mut log_file = std::fs::File::create("nestest_output.log").unwrap();
        let mut cycles_since_sample: f64 = 0.0;

        let mut sample_count = 0;
        let mut sample_sum = 0.0;
        
        if is_key_pressed(KeyCode::Space) {
            emulator.is_paused = !emulator.is_paused;
        }
        if !emulator.is_paused {
            emulator.cpu.bus.ppu.frame_complete = false;
            let sample = emulator.cpu.bus.apu.get_sample();
            if sample != 0.0 {
                //println!("Opa, tem som saindo da APU: {}", sample);
            }
            while !emulator.cpu.bus.ppu.frame_complete {
                //emulator.cpu.log_state(&mut log_file);
                emulator.step(audio);
            }
        }

        //at the end of each step, takes the frame buffer and updates the ppu texture with it, therefore rendering a new frame
        emulator.image.bytes.copy_from_slice(&emulator.cpu.bus.ppu.frame_buffer);
        emulator.ppu_texture.update(&emulator.image);


        draw_texture_ex(
            &emulator.ppu_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(256.0 * (2.0 * MULTIPLY_RESOLUTION as f32), 240.0 * (2.0 * MULTIPLY_RESOLUTION as f32))), 
                ..Default::default()
            },
        );
        
        if is_key_pressed(KeyCode::F1) {
            //TODO ver um jeito de ajustar para escalar a janela para o tamanho certo
            // ou simplesmente colocar uma tela preta no lugar das infos quando eu esconder elas
            emulator.show_debug_info = !emulator.show_debug_info;
        }

        draw_text(&get_fps().to_string(), 10.0, 20.0, 30.0, WHITE);
    
        if emulator.show_debug_info {
            let pos_x: f32 = 520.0 * MULTIPLY_RESOLUTION as f32; // Posição X para informações de depuração
            let mut pos_y: f32 = 30.0; // Posição Y inicial
            let line_height = 30.0; // Altura da linha para espaçamento
            let font_size = 30.0; // Tamanho da fonte

            //volume: 
            draw_text(&format!("Vol: {:.0}%", emulator.cpu.bus.apu.volume * 100.0), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height * 2.0;

            // cpu info
            draw_text(&format!("STATUS: {}", CPU::format_cpu_status(emulator.cpu.status.bits())), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PC: {:#06x}", emulator.cpu.program_counter), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("CYCLES: {:?}", emulator.cpu.cycles), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("A: {:#04x} | X: {:#04x} | Y: {:#04x}", emulator.cpu.register_a, emulator.cpu.register_x, emulator.cpu.register_y), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height * 2.0;

            // ppu info
            draw_text(&format!("PPU INFO:"), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUCTRL: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.ctrl.bits(), emulator.cpu.bus.ppu.ctrl.bits()), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUMASK: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.mask, emulator.cpu.bus.ppu.mask), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUSTATUS: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.status.bits(), emulator.cpu.bus.ppu.status.bits()), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("TempVRAM: {:#06x}", emulator.cpu.bus.ppu.t.addr), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("VRAM: {:#06x}", emulator.cpu.bus.ppu.t.addr), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPU Cycle: {} | Scanline: {}", emulator.cpu.bus.ppu.cycle, emulator.cpu.bus.ppu.scanline), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("Frame Complete: {:?}", emulator.cpu.bus.ppu.frame_complete), pos_x, pos_y, font_size, WHITE);

        }
    }

    loop {
        match &mut state {
            EmulatorState::Menu => {
                clear_background(DARKBLUE);

                //USER INPUT
                let default_game_message = format!("Pressione 1 para carregar o jogo padrão ({})", DEFAULT_GAME_NAME);
                draw_text( &default_game_message, 20.0, 50.0, 30.0, WHITE);
                draw_text("Pressione '2' para digitar outro caminho.", 20.0, 120.0, 30.0, WHITE);

                if is_key_pressed(KeyCode::Key1) {
                    println!("Iniciando jogo Padrão");
                    state = EmulatorState::Loading { game_path: PathBuf::from(DEFAULT_GAME_FILE) }
                } else if is_key_pressed(KeyCode::Key2) {
                    while get_char_pressed().is_some() { }

                    state = EmulatorState::TypingPath;
                }
            }
            EmulatorState::TypingPath => {
                clear_background(DARKBLUE);

                draw_text("Cole ou digite o caminho do arquivo:", 20.0, 50.0, 40.0, BLACK);

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
                draw_text("Pressione ENTER para confirmar ou ESC para cancelar", 20.0, 150.0, 40.0, BLACK);
            }

            EmulatorState::Loading { game_path } => {
                draw_text("CARREGANDO...", 200.0, 200.0, 50.0, YELLOW);
                
                let emu_instance = EmulatorInstance::new(game_path.to_path_buf());
                let audio = AudioOutput::new(44100);

                //print_program(&emu_instance);
                println!("tipo de mirroring: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
                
                state = EmulatorState::Running { emulator_instance: emu_instance, audio };
            }
            EmulatorState::Running { ref mut emulator_instance, ref mut audio } => {

                emulator_instance.cpu.bus.joypad_1.set_button(
                    JoyPadButtons::A, is_key_down(KeyCode::J) || is_key_down(KeyCode::X) 
                );
                emulator_instance.cpu.bus.joypad_1.set_button(
                    JoyPadButtons::B, is_key_down(KeyCode::K) || is_key_down(KeyCode::C)
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

                rungame(emulator_instance, audio).await;

                //cant change state beffore droping borrow, so it must be after rungame
                if is_key_pressed(KeyCode::Escape) {
                    state = EmulatorState::Menu;
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