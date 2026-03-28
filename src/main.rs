use std::{fmt::Error, io, path::{Path, PathBuf}};

use macroquad::{prelude::*, ui::{hash, root_ui, widgets}};
use arboard::Clipboard;
use sysinfo::{System, Pid};

use crate::{
    cpu::cpu::CPU, 
    memory::joypads::JoyPadButtons,
    apu::audio::AudioOutput,
    menu::menu::*,
};

mod cpu;
mod memory;
mod ppu;
mod apu;
mod menu;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;


const MULTIPLY_RESOLUTION: i32 = 2;

const DEFAULT_GAME_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/NES_GAMES/Mario/Super Mario Bros. (World).nes");
const DEFAULT_GAME_NAME: &str = "Super Mario Bros. (World)";

struct PerfomanceStats {
    sys: System,
    pid: Pid,

    cpu_usage: f32,
    main_emu_thread: f32,
    memory_usage_mb: f64,
    last_update: f64,
}
impl PerfomanceStats {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
            pid: sysinfo::get_current_pid().unwrap(),
            cpu_usage: 0.0,
            main_emu_thread: 0.0,
            memory_usage_mb: 0.0,
            last_update: get_time(),
        }
    }
    pub fn update_status(&mut self) {
        let current_time = get_time();
        if current_time - self.last_update >= 0.5 {
            self.sys.refresh_process(self.pid);
            if let Some(process) = self.sys.process(self.pid) {
                self.main_emu_thread = process.cpu_usage(); 
                self.cpu_usage = self.main_emu_thread / self.sys.cpus().len() as f32;
                self.memory_usage_mb = process.memory() as f64 / 1024.0 / 1024.0;
            } 
            self.last_update = current_time;
        }
    }
}

struct EmulatorInstance {
    //emulator itself
    cpu: CPU, 

    image: Image, 
    ppu_texture: Texture2D, 

    show_debug_info: bool, 
    is_paused: bool,

    debug_frame_counter: u8,
    cached_debug_text: Vec<String>,

    stats: PerfomanceStats

} impl EmulatorInstance {
    fn new(game_path: PathBuf) -> Result<EmulatorInstance, Box<dyn std::error::Error>> {
        let mapper = memory::bus::load_rom_from_file(Path::new(game_path.as_path()))?;
        
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

        Ok(EmulatorInstance { 
            //mapper: mapper,
            cpu: cpu, 
            image: image, 
            ppu_texture: ppu_texture, 
            show_debug_info: false, 
            is_paused: false,
            stats: PerfomanceStats::new(),

            debug_frame_counter: 0,
            cached_debug_text: Vec::new(),
        })
    }    
    pub fn show_debug_info(&mut self) {
        if self.show_debug_info {
            let pos_x: f32 = 520.0 * MULTIPLY_RESOLUTION as f32; // Posição X para informações de depuração
            let mut pos_y: f32 = 30.0; // Posição Y inicial
            let line_height = 30.0; // Altura da linha para espaçamento
            let font_size = 30.0; // Tamanho da fonte

            self.debug_frame_counter = (self.debug_frame_counter + 1) % 4;

            if self.debug_frame_counter == 0 {

                self.stats.update_status();

                self.cached_debug_text.clear();

                // Geral
                self.cached_debug_text.push(format!("Vol: {:.0}%", self.cpu.bus.apu.volume * 100.0));
                self.cached_debug_text.push(format!("CPU: {:.1}% | Thread: {:.1}%", self.stats.cpu_usage, self.stats.main_emu_thread));
                self.cached_debug_text.push(format!("RAM: {:.2} MB", self.stats.memory_usage_mb));

                // Espaçamento vazio (opcional, para manter o layout original)
                self.cached_debug_text.push(String::new());

                // CPU Info
                self.cached_debug_text.push(format!("STATUS: {}", CPU::format_cpu_status(self.cpu.status.bits())));
                self.cached_debug_text.push(format!("PC: {:#06x}", self.cpu.program_counter));
                self.cached_debug_text.push(format!("CYCLES: {:?}", self.cpu.cycles));
                self.cached_debug_text.push(format!("A: {:#04x} | X: {:#04x} | Y: {:#04x}", self.cpu.register_a, self.cpu.register_x, self.cpu.register_y));

                // Espaçamento
                self.cached_debug_text.push(String::new());

                // PPU Info
                self.cached_debug_text.push(String::from("PPU INFO:"));
                self.cached_debug_text.push(format!("PPUCTRL: {:#010b} ({:#04x})", self.cpu.bus.ppu.ctrl.bits(), self.cpu.bus.ppu.ctrl.bits()));
                self.cached_debug_text.push(format!("PPUMASK: {:#010b} ({:#04x})", self.cpu.bus.ppu.mask, self.cpu.bus.ppu.mask));
                self.cached_debug_text.push(format!("PPUSTATUS: {:#010b} ({:#04x})", self.cpu.bus.ppu.status.bits(), self.cpu.bus.ppu.status.bits()));
                self.cached_debug_text.push(format!("TempVRAM: {:#06x}", self.cpu.bus.ppu.t.addr));
                self.cached_debug_text.push(format!("VRAM: {:#06x}", self.cpu.bus.ppu.t.addr));
                self.cached_debug_text.push(format!("PPU Cycle: {} | Scanline: {}", self.cpu.bus.ppu.cycle, self.cpu.bus.ppu.scanline));
                self.cached_debug_text.push(format!("Frame Complete: {:?}", self.cpu.bus.ppu.frame_complete));
            }

            for line in &self.cached_debug_text {
                draw_text(line, pos_x, pos_y, font_size, WHITE);
                pos_y += line_height;
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

    let frame_time = std::time::Duration::from_secs_f64(1.0/60.0);
    let mut frame_deadline = std::time::Instant::now();
    let skin = create_customized_skin(MULTIPLY_RESOLUTION as f32);

    async fn rungame(emulator: &mut EmulatorInstance, audio: &mut Option<(AudioOutput, u32)>) {

        //let mut log_file = std::fs::File::create("nestest_output.log").unwrap();
        //let mut cycles_since_sample: f64 = 0.0;

        //let mut sample_count = 0;
        //let mut sample_sum = 0.0;

        emulator.cpu.bus.ppu.frame_complete = false;

        while !emulator.cpu.bus.ppu.frame_complete {
            //emulator.cpu.log_state(&mut log_file);
            let (_, cycles) = emulator.cpu.step();

            if let Some(ref mut audio) = audio {
                emulator.cpu.bus.sync_audio(cycles, audio);
            }
        }

        //at the end of each step, takes the frame buffer and updates the ppu texture with it, therefore rendering a new frame
        emulator.image.bytes.copy_from_slice(&emulator.cpu.bus.ppu.frame_buffer);
        emulator.ppu_texture.update(&emulator.image);

        draw_texture_ex(
            &emulator.ppu_texture,
            0.0,
            0.0,
            WHITE, /* Color { r: 1.0, g: 0.95, b: 0.95, a: 1.0 } */
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
        if is_key_pressed(KeyCode::Period) {
            emulator.cpu.bus.ppu.color_palette.cycle_palettes();
        }

        draw_text(&get_fps().to_string(), 10.0, 20.0, 30.0, WHITE);
        emulator.show_debug_info();
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
                
                let emu_instance = match EmulatorInstance::new(game_path.to_path_buf()) {
                    Ok(emu) => emu,
                    Err(err) => {
                        eprint!("[ERROR] An error occoured while Loading the ROM: {}", err);
                        state = EmulatorState::Menu;
                        path_buffer.clear();
                        continue;
                    }
                };

                let audio = AudioOutput::new(44100);

                //print_program(&emu_instance);
                println!("tipo de mirroring: {:?}", emu_instance.cpu.bus.ppu.ppubus.mapper.borrow().mirroring());
                
                state = EmulatorState::Running { emulator_instance: emu_instance, audio};
            }
            EmulatorState::Running { ref mut emulator_instance, ref mut audio } => {

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

                if !emulator_instance.is_paused {
                    rungame(emulator_instance, audio).await;
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