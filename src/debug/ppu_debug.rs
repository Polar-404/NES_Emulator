use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Result};
use std::io::Write;

use crate::cpu::cpu::CPU;
use crate::debug::cpu_debug::LoggerGuard;

struct PpuSnapshot {
    scanline: i16,
    cycle: u16,
    v_addr: u16,
    bg_netx_tile: u8,
}

impl Display for PpuSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "SCANLINE:{:04X} CYC:{:04X} V_ADDR:{:04X} BG_N_T_ID:{:04X}",
            self.scanline, self.cycle, self.v_addr, self.bg_netx_tile
        )
    }
}

#[allow(unused)]
pub fn log_ppu(
    path: Option<&str>,
    size: usize,
    mut stop_condition: impl FnMut(&mut CPU) -> bool + 'static,
) -> impl FnMut(&mut CPU) + 'static {
    
    let mut guard = LoggerGuard::new(
        VecDeque::with_capacity(size),
        path.unwrap_or("ppu_log.txt").to_string()
    );

    move |cpu: &mut CPU| {

        let snapshot = PpuSnapshot {
            v_addr: cpu.bus.ppu.v.addr,
            bg_netx_tile: cpu.bus.ppu.bg_next_tile_id,
            scanline: cpu.bus.ppu.scanline,
            cycle: cpu.bus.ppu.cycle
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