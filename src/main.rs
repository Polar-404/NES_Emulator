use std::{env, path::PathBuf, process::Command};

use macroquad::prelude::*;


const DEFAULT_GAME_FILE: &'static str = "C:/Users/migue/OneDrive/Documents/CODEGO/Rust/NES_Emulador/NES_GAMES/Mario/Super Mario Bros. (World).nes";

fn main() {
    //let mapper = memory::bus::load_rom_from_file(Path::new("NES_GAMES/Crisis Force (Japan).nes"));
    //let mapper = memory::bus::load_rom_from_file(Path::new("NES_GAMES/Ms. Pac-Man (USA) (Tengen) (Unl).nes"));
    //let mapper = memory::bus::load_rom_from_file(Path::new("NES_GAMES/Nuts & Milk (Japan).nes"));

    let game_file: PathBuf;

    if get_game(DEFAULT_GAME_FILE) {
        game_file = PathBuf::from(DEFAULT_GAME_FILE);
    } else {
        println!("Digite o caminho para o arquivo do jogo:");
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input).expect("falha ao ler o caminho do jogo");
        game_file = PathBuf::from(user_input.trim());
    }

    let mut exec_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("emulator_loop");

    if cfg!(target_os = "windows") {
        exec_path.set_extension("exe");
    }

    let status = Command::new(exec_path)
        .arg(&game_file)
        .status()
        .expect("Falha ao iniciar o emulador.");
    if status.success() {
        println!("Emulador finalizado com sucesso.");
    } else {
        println!("Emulador finalizado com erro.");
    }
}

fn get_game(game_file: &str) -> bool {
    // Lógica de entrada do usuário...
    println!("Escolha uma opção:");
    println!("Usar o jogo padrão ({})? [y / n]", game_file);
    
    loop {
        let mut use_game_option: String = String::new();
        std::io::stdin().read_line(&mut use_game_option).expect("falha ao ler a mensagem");
        
        match use_game_option.trim().to_lowercase().as_str() {
            "1" | "y" | "s" | "sim" => return true,
            "0" | "n" | "nao" | "não" => return false,
            _ => {
                println!("Entrada inválida. Por favor, digite 'y' ou 'n'.");
                continue
            }
        }
    }
}