const TRIANGLE_SEQUENCE: [f32; 32] = [
    15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0,
    7.0,  6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
    0.0,  1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0,
    8.0,  9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
];

#[derive(Debug, Default)]
pub struct TriangleWave {
    pub enabled: bool,
    timer_value: u16,
    timer_reload: u16,
    linear_counter: u8,
    counter_reload: u8,
    sequence_pos: u8,
    linear_halt: bool,
    linear_reload_flag: bool,
    length_counter: u8,
}

impl TriangleWave {
    pub fn step(&mut self) {
        if !self.enabled { return }

        if self.linear_reload_flag {
            self.linear_counter = self.counter_reload;
            if !self.linear_halt {
                self.linear_reload_flag = false;
            }
        }

        if self.linear_counter == 0 || self.length_counter == 0 { return }

        if self.timer_value == 0 {
            self.timer_value = self.timer_reload + 1;
            self.sequence_pos = (self.sequence_pos + 1) % 32
        } else {
            self.timer_value -= 1;
        }
    }
    
    pub fn get_amplitude(&mut self) -> f32 {
        if self.length_counter == 0 || self.linear_counter == 0 || !self.enabled { return 0.0 }
        let sample = TRIANGLE_SEQUENCE[self.sequence_pos as usize];
        sample
    }

    //240Hz (quarter frame)
    pub fn clock_linear_counter(&mut self) {
        if self.linear_reload_flag {
            self.linear_counter = self.counter_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        
        if !self.linear_halt {
            self.linear_reload_flag = false;
        }
    }

    // 120Hz (half Frame)
    pub fn clock_length(&mut self) {
        if !self.linear_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
    ///CRRR RRRR (C == Halt)
    pub fn write_linear_counter(&mut self, data: u8) {
        self.counter_reload = data & 0x7F;
        self.linear_halt = (data >> 7) != 0;
    }
    pub fn write_timer_lo(&mut self, data: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | data as u16
    }
    pub fn write_timer_hi(&mut self, data: u8) {
        self.timer_reload = (self.timer_reload & 0x00FF) | (data as u16 & 0b111) << 8;
        self.linear_reload_flag = true;
        self.length_counter = super::square::LENGTH_TABLE[(data >> 3) as usize];
    }
}