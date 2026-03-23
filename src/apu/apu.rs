use crate::apu::square::SquareWave;

use super::{square};

/// NES Audio Processing Unit
/// 
/// for more info:
/// https://www.nesdev.org/wiki/APU
#[derive(Default, Debug)]
pub struct APU {
    pulse1: SquareWave,
    pulse2: SquareWave,
}
impl APU {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            //pulse 1
            0x4000 => self.pulse1.write_control(data),
            0x4001 => {/* https://www.nesdev.org/wiki/APU_Sweep */},
            0x4002 => self.pulse1.write_timer_lo(data),
            0x4003 => self.pulse1.write_timer_hi(data),

            0x4004 => self.pulse1.write_control(data),
            0x4005 => {/* https://www.nesdev.org/wiki/APU_Sweep */},
            0x4006 => self.pulse1.write_timer_lo(data),
            0x4007 => self.pulse1.write_timer_hi(data),

            0x4015 => {
                self.pulse1.enabled = data & 0x01 != 0;
                self.pulse2.enabled = data & 0x02 != 0;
            }
            _ => {}
        }
    }

    pub fn step(&mut self) {
        self.pulse1.step();
        self.pulse2.step();
    }

    pub fn get_sample(&mut self) -> f32 {
        let p1 = self.pulse1.get_amplitude();
        let p2 = self.pulse2.get_amplitude();

        (p1 + p2) * 0.5
    }

    pub fn read_status(&self) -> u8 {
        0
    }
}
