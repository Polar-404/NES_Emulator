use std::{path::{Path, PathBuf}};

use macroquad::prelude::*;
use arboard::Clipboard;

use crate::{cpu::cpu::CPU};
mod cpu;
mod memory;
mod ppu;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;


const MULTIPLY_RESOLUTION: i32 = 2;

const DEFAULT_GAME_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/NES_GAMES/Mario/Super Mario Bros. (World).nes");

struct EmulatorInstance {
    //mapper: Rc<std::cell::RefCell<Box<dyn Mapper + 'static>>>,  ( todo! talvez n seja necessario esse mapper)
    cpu: CPU, 
    image: Image, 
    ppu_texture: Texture2D, 
    show_debug_info: bool, 
    is_paused: bool
} impl EmulatorInstance {
    fn new(game_path: PathBuf) -> EmulatorInstance {
        
        let mapper = memory::bus::load_rom_from_file(Path::new(game_path.as_path()));
        
        let mut cpu = CPU::new(mapper);
        
        cpu.reset_interrupt();

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
            is_paused: false
        }
    }
}
enum EmulatorState {
    Menu,

    TypingPath,

    Loading { game_path: PathBuf },

    Running { emulator_instance: EmulatorInstance }
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

    async fn rungame(emulator: &mut EmulatorInstance) {
        if is_key_pressed(KeyCode::Space) {
            emulator.is_paused = !emulator.is_paused;
        }
        if !emulator.is_paused {
            emulator.cpu.bus.ppu.frame_complete = false;

            //for _ in 0..100 {
            //    emulator.cpu.step(|_| {});
            //}

            while !emulator.cpu.bus.ppu.frame_complete {
                emulator.cpu.step(|_| {});
            }
        }

        if emulator.cpu.bus.ppu.frame_complete {
            //at the end of each step, takes the frame buffer and updates the ppu texture with it, therefore rendering a new frame
            emulator.image.bytes.copy_from_slice(&emulator.cpu.bus.ppu.frame_buffer);
            emulator.ppu_texture.update(&emulator.image);
        }

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
    
        if emulator.show_debug_info {
            let pos_x: f32 = 520.0 * MULTIPLY_RESOLUTION as f32; // Posição X para informações de depuração
            let mut pos_y: f32 = 30.0; // Posição Y inicial
            let line_height = 30.0; // Altura da linha para espaçamento
            let font_size = 30.0; // Tamanho da fonte

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
            draw_text(&format!("PPUCTRL: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.ppu_ctrl.bits(), emulator.cpu.bus.ppu.ppu_ctrl.bits()), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUMASK: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.ppu_mask, emulator.cpu.bus.ppu.ppu_mask), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUSTATUS: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.ppu_status.bits(), emulator.cpu.bus.ppu.ppu_status.bits()), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("OAMADDR: {:#04x}", emulator.cpu.bus.ppu.oam_addr), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUADDR (VRAM): {:#06x}", emulator.cpu.bus.ppu.ppu_addr.value), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPU Cycle: {} | Scanline: {}", emulator.cpu.bus.ppu.cycle, emulator.cpu.bus.ppu.scanline), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPU STATUS: {:?}", emulator.cpu.bus.ppu.format_ppu_status(emulator.cpu.bus.ppu.ppu_status.bits())), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("Frame Complete: {:?}", emulator.cpu.bus.ppu.frame_complete), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("last read palette: {:#010b} ({:#04x})", emulator.cpu.bus.ppu.ppubus.last_read_palette, emulator.cpu.bus.ppu.ppubus.last_read_palette), pos_x, pos_y, font_size, WHITE);
        }
    }

    loop {
        match &mut state {
            EmulatorState::Menu => {
                clear_background(DARKBLUE);

                //USER INPUT
                let default_game_message = format!("Pressione 1 para carregar o jogo padrão ({})", DEFAULT_GAME_FILE);
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

                print_program(&emu_instance);
                println!("tipo de mirroring: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
                
                state = EmulatorState::Running { emulator_instance: emu_instance };
            }
            EmulatorState::Running { ref mut emulator_instance } => {
                rungame(emulator_instance).await;
            }
        }

        next_frame().await

    }
}

fn print_program(emulator: &EmulatorInstance) {
    for addr in (0x0000..=0x1FFF).step_by(16) {
        print!("{:04X}: ", addr);
        for offset in 0..16 {
            let byte = emulator.cpu.bus.ppu.ppubus.mapper.borrow().read_chr(addr + offset);
            print!("{:02X} ", byte);
        }
        println!();
    }
}