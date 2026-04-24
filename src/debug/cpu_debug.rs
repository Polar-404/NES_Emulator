use std::{
    fmt::{Display, Formatter, Result},
    io::{
        Write,
    }
};

use std::collections::VecDeque;
use crate::cpu::cpu::CPU;

#[derive(Clone, Copy)]
struct CpuSnapshot {
    pc: u16,
    opcode: u8,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    st: u8,
    cyc: u64,
}

impl Display for CpuSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "PC:{:04X} OP:{:02X} A:{:02X} X:{:02X} Y:{:02X} SP:{:02X} ST:{:02X} CYC:{}",
            self.pc, self.opcode, self.a, self.x, self.y, self.sp, self.st, self.cyc
        )
    }
}
pub struct LoggerGuard<T: Display> {
    pub buffer: VecDeque<T>,
    pub path: String,
}

impl<T: Display> LoggerGuard<T> {
    pub fn new(buffer: VecDeque<T>, path: String,) -> Self {
        LoggerGuard {
            buffer,
            path,
        }
    }
}

impl<T: Display> Drop for LoggerGuard<T> {
    fn drop(&mut self) {
        if let Ok(file) = std::fs::File::create(&self.path) {
            let mut writer = std::io::BufWriter::new(file);
            for item in &self.buffer {
                let _ = writeln!(writer, "{}", item);
            }
            let _ = writer.flush();
            println!("Logs gravados em {} (via Drop)", self.path);
        }
    }
}


/// The function is specialy made to be used as a callback to the CPU ```"step_with_callback()"``` func
/// 
/// It saves most of it's imporrtant info such as reigster values and program counter, stack_pointer. status flags, and current cycle
/// 
/// Writes it all on a circular buffer which it's **```size```** must be given as a parameter, 
/// that starts popping the oldest data once it gets full
/// 
/// If no file path was given it will create a default one 'cpu_log.txt' at the root of the project
/// 
/// 
#[allow(unused)]
pub fn cpu_logger(
    path: Option<&str>,
    size: usize,
    mut stop_condition: impl FnMut(&mut CPU) -> bool + 'static,
) -> impl FnMut(&mut CPU) + 'static {

    let mut guard = LoggerGuard {
        path: path.unwrap_or("cpu_log.txt").to_string(),
        buffer: VecDeque::with_capacity(size)
    };

    move |cpu: &mut CPU| {

        let snapshot = CpuSnapshot {
            pc: cpu.program_counter,
            opcode: cpu.last_opcode,
            a: cpu.register_a,
            x: cpu.register_x,
            y: cpu.register_y,
            sp: cpu.stack_pointer,
            st: cpu.status.bits(),
            cyc: cpu.cycles,
        };

        if guard.buffer.len() == size {
            guard.buffer.pop_front();
        }
        guard.buffer.push_back(snapshot);

        if stop_condition(cpu) {
            panic!("Stop Condition met at (PC:{:04X} | Instruction:{:04X}).", cpu.program_counter, cpu.last_opcode);
        }
    }
}


/// function to log the cpu state with the nestest.nes log format
#[allow(dead_code)]
pub fn log_state_nestest(
    path: Option<&str>,
    size: usize,
    mut stop_condition: impl FnMut(&mut CPU) -> bool + 'static,
) -> impl FnMut(&mut CPU) + 'static {
        
    let mut guard = LoggerGuard {
        path: path.unwrap_or("cpu_log.txt").to_string(),
        buffer: VecDeque::with_capacity(size)
    };

    move |cpu: &mut CPU| {

        let pc = cpu.program_counter;
        let opcode = cpu.last_opcode;
        
        let line = format!(
            "{:04X}  {:02X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            pc, opcode, cpu.register_a, cpu.register_x, cpu.register_y, 
            cpu.status.bits(), cpu.stack_pointer, cpu.cycles
        );

        if guard.buffer.len() >= size {
            guard.buffer.pop_front();
        }
        guard.buffer.push_back(line);

        if stop_condition(cpu) {
            panic!("Stop Condition met at (PC:{:04X} | Instruction:{:04X}).", cpu.program_counter, cpu.last_opcode);
        }
    }
}

#[allow(unused)] //TODO
fn opcode_size(opcode: u8) -> u8 {
    match opcode {
        // size 1
        0x00 | 0x08 | 0x18 | 0x28 | 0x38 | 0x40 | 0x48 | 0x58 |
        0x60 | 0x68 | 0x78 | 0x88 | 0x8A | 0x98 | 0x9A | 0xA8 |
        0xAA | 0xB8 | 0xBA | 0xC8 | 0xCA | 0xD8 | 0xE8 | 0xEA |
        0xF8 | 0x0A | 0x2A | 0x4A | 0x6A => 1,
        // size 3
        0x0D | 0x0E | 0x19 | 0x1D | 0x20 | 0x2C | 0x2D | 0x2E |
        0x39 | 0x3D | 0x4C | 0x4D | 0x4E | 0x59 | 0x5D | 0x6D |
        0x6E | 0x79 | 0x7D | 0x8C | 0x8D | 0x8E | 0x99 | 0x9D |
        0xAC | 0xAD | 0xAE | 0xB9 | 0xBC | 0xBD | 0xBE | 0xCC |
        0xCD | 0xCE | 0xD9 | 0xDD | 0xEC | 0xED | 0xEE | 0xF9 |
        0xFD => 3,
        _ => 2,
    }
}