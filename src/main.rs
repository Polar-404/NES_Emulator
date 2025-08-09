pub mod cpu;
pub mod opcodes;
pub mod bus;
pub mod dummy_mapper;

use macroquad::prelude::*;

use crate::{cpu::CPU};

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

#[macroquad::main("My NES Emulator")]
async fn main() {

    let mut program: Vec<u8> = vec![0; 0x1FFF];
    program[0x0300] = 0xA9; // LDA #$0A
    program[0x0301] = 0x0A;
    program[0x0302] = 0x85; // STA $00
    program[0x0303] = 0x00;
    program[0x0304] = 0x4C; // JMP $C000
    program[0x0305] = 0x00;
    program[0x0306] = 0xC0;
    program[0x03FC] = 0x00; // Vetor de reset para 0xC000
    program[0x03FD] = 0xC0;


    let mapper = dummy_mapper::TestMapper::new(program);
    let mut cpu = CPU::new(mapper);
    
    cpu.reset_interrupt();
    debbuger_info(&mut cpu).await

}
async fn debbuger_info(cpu: &mut CPU) {
    let mut show_debug_info = true;


    loop {
        clear_background(BLUE);
        cpu.step(|_| {});
        
        if is_key_pressed(KeyCode::F1) {
            show_debug_info = !show_debug_info;
        }
    
        if show_debug_info {
            let pos_x: f32 = 100.0;
            let pos_y: f32 = 100.0;
            draw_text(&format!("STATUS: {:?}", cpu.status), pos_x, pos_y, 20.0, WHITE);
            draw_text(&format!("PC: {:#06x}", cpu.program_counter), pos_x, pos_y + 20.0, 20.0, WHITE);
            draw_text(&format!("Register A: {:#06x}", cpu.register_a), pos_x, pos_y + 40.0, 20.0, WHITE);
            draw_text(&format!("Register X: {:#06x}", cpu.register_x), pos_x, pos_y + 60.0, 20.0, WHITE);
            draw_text(&format!("Register Y: {:#06x}", cpu.register_y), pos_x, pos_y + 80.0, 20.0, WHITE);
        }

        next_frame().await
    }

}