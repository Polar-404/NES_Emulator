use std::path::Path;

use macroquad::prelude::*;

use crate::cpu::cpu::CPU;
mod cpu;
mod memory;
mod ppu;


#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;


const MULTIPLY: i32 = 2;

fn window_conf() -> Conf {
    Conf {
        window_title: "NES Emulator".to_owned(),

        window_width: 720 * MULTIPLY,
        window_height: 480 * MULTIPLY,

        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let game_file = &args[1];
    
    let mapper = memory::bus::load_rom_from_file(Path::new(game_file));
    
    let mut cpu = CPU::new(mapper);
    
    cpu.reset_interrupt();
    
    let mut show_debug_info = true;

    let mut image = Image::gen_image_color(256, 240, Color { r: 0.0 , g: 0.0, b: 0.0, a: 1.0 });
    let ppu_texture = Texture2D::from_image(&image);

    ppu_texture.set_filter(FilterMode::Nearest);
    
    loop {
        clear_background(BLACK);

        //TODO lembrar de colocar isso aqui de volta depois
        //while !cpu.bus.ppu.frame_complete {
        //    cpu.step(|_| {});
        //}

        for _ in 0..=100 {
            cpu.step(|_| {});
        }

        //cpu.step(|_| {});
        
        if cpu.bus.ppu.frame_complete {
            //at the end of each step, takes the frame buffer and updates the ppu texture with it, therefore rendering a new frame
            image.bytes.copy_from_slice(&cpu.bus.ppu.frame_buffer);
            ppu_texture.update(&image);
        }

        // Desenhe a textura da PPU na tela
        // Você pode ajustar a posição e o tamanho (dest_size) conforme necessário
        draw_texture_ex(
            &ppu_texture,
            0.0,
            0.0,
            DARKGRAY,
            DrawTextureParams {
                dest_size: Some(vec2(256.0 * (2.0 * MULTIPLY as f32), 240.0 * (2.0 * MULTIPLY as f32))), 
                ..Default::default()
            },
        );
        
        if is_key_pressed(KeyCode::F1) {
            //TODO ver um jeito de ajustar para escalar a janela para o tamanho certo
            // ou simplesmente colocar uma tela preta no lugar das infos quando eu esconder elas
            show_debug_info = !show_debug_info;
        }
    
        if show_debug_info {
            let pos_x: f32 = 520.0 * MULTIPLY as f32; // Posição X para informações de depuração
            let mut pos_y: f32 = 30.0; // Posição Y inicial
            let line_height = 30.0; // Altura da linha para espaçamento
            let font_size = 30.0; // Tamanho da fonte

            // Dentro da sua função debbuger_info
            draw_text(&format!("STATUS: {}", CPU::format_cpu_status(cpu.status.bits())), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PC: {:#06x}", cpu.program_counter), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("CYCLES: {:?}", cpu.cycles), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("A: {:#04x} | X: {:#04x} | Y: {:#04x}", cpu.register_a, cpu.register_x, cpu.register_y), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height * 2.0;

            // Exemplo de informações da PPU
            draw_text(&format!("PPU INFO:"), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUCTRL: {:#010b} ({:#04x})", cpu.bus.ppu.ppu_ctrl.bits(), cpu.bus.ppu.ppu_ctrl.bits()), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUMASK: {:#010b} ({:#04x})", cpu.bus.ppu.ppu_mask, cpu.bus.ppu.ppu_mask), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUSTATUS: {:#010b} ({:#04x})", cpu.bus.ppu.ppu_status.bits(), cpu.bus.ppu.ppu_status.bits()), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("OAMADDR: {:#04x}", cpu.bus.ppu.oam_addr), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPUADDR (VRAM): {:#06x}", cpu.bus.ppu.ppu_addr.value), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPU Cycle: {} | Scanline: {}", cpu.bus.ppu.cycle, cpu.bus.ppu.scanline), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("PPU STATUS: {:?}", cpu.bus.ppu.format_ppu_status(cpu.bus.ppu.ppu_status.bits())), pos_x, pos_y, font_size, WHITE);
            pos_y += line_height;
            draw_text(&format!("Frame Complete: {:?}", cpu.bus.ppu.frame_complete), pos_x, pos_y, font_size, WHITE);
        }

        next_frame().await
    }

}