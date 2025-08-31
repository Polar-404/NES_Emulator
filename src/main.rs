mod cpu;
mod memory;
mod ppu;

use std::{env, path::Path, process::Command};

use macroquad::prelude::*;
//use memory::dummy_mapper::TestMapper;

use crate::{cpu::cpu::CPU};

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;


const DEFAULT_GAME_FILE: &'static str = "NES_GAMES/Mario/Super Mario Bros. (World).nes";


fn main() {
    //let mapper = memory::bus::load_rom_from_file(Path::new("NES_GAMES/Crisis Force (Japan).nes"));
    //let mapper = memory::bus::load_rom_from_file(Path::new("NES_GAMES/Ms. Pac-Man (USA) (Tengen) (Unl).nes"));
    //let mapper = memory::bus::load_rom_from_file(Path::new("NES_GAMES/Nuts & Milk (Japan).nes"));

    let mut game_file = DEFAULT_GAME_FILE;

    let default_game = get_game(game_file);

    if !default_game {
        println!("Digite o caminho para o arquivo do jogo:");
        std::io::stdin().read_line(&mut game_file.to_string()).expect("falha ao ler o caminho do jogo");
        game_file = game_file.trim();
    }

    // Obtenha o caminho para o executável `gui_runner`

    Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("emulator_loop")
        .arg(&game_file)
        .spawn()
        .expect("Falha ao iniciar o emulador.");
}

fn get_game(game_file: &str) -> bool {
    // Lógica de entrada do usuário...
    println!("Escolha uma opção:");
    println!("Usar o jogo padrão ({})? [y / n]", game_file);
    
    loop {
        let mut use_game_option: String = String::new();
        std::io::stdin().read_line(&mut use_game_option).expect("falha ao ler a mensagem");
        
        match use_game_option.trim() {
            "1" | "y" => return true,
            "0" | "n" => return false,
            _ => {
                println!("Entrada inválida. Por favor, digite 1 ou 0.");
                continue
            }
        }
    }
}

//unused loop
    // let mut program: Vec<u8> = vec![0; 0x1FFF];
    // program[0x0300] = 0xA9; // LDA #$0A
    // program[0x0301] = 0x0A;
    // program[0x0302] = 0x85; // STA $00
    // program[0x0303] = 0x00;
    // program[0x0304] = 0x4C; // JMP $C000
    // program[0x0305] = 0xff;
    // program[0x0306] = 0x00;
    // program[0x03FC] = 0x00; // Vetor de reset para 0xC000
    // program[0x03FD] = 0xC0;