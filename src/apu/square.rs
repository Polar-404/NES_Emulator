
const SEQUENCE: [[bool; 8]; 4] = [
    [false, true, false, false, false, false, false, false], // 12.5%
    [false, true, true, false, false, false, false, false],  // 25%
    [false, true, true, true, true, false, false, false],   // 50%
    [true, false, false, true, true, true, true, true],     // 75% (25% invertido)
];

#[derive(Default, Debug)]
pub struct SquareWave {
    pub enabled:    bool,
    timer_reload:   u16,
    timer_value:    u16,
    duty_cycle:     u8,
    duty_value:     u8,
    volume:         u8
}

impl SquareWave {
    pub fn new(&mut self) -> Self {
        Self::default()
    }
    pub fn step(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_reload;
            self.duty_value = (self.duty_value + 1) % 8;
        } else {
            self.timer_value -= 1;
        }
    }
    pub fn get_amplitude(&self) -> f32 {
        if !self.enabled { return 0.0 }
        let sample: f32 = if SEQUENCE[self.duty_cycle as usize][self.duty_value as usize] { 1.0 } else { -1.0 };
        sample * (self.volume as f32 / 15.0)
    }
    pub fn write_timer_lo(&mut self, data: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | data as u16
    } 
    pub fn write_timer_hi(&mut self, data: u8) {
        let data = data & 0b111;
        self.timer_reload = (self.timer_reload & 0x00FF) | ((data as u16) << 8);
        self.duty_value = 0;
    }
    pub fn write_control(&mut self, data: u8) {
        self.duty_cycle = data >> 6 & 0b11;
        self.volume = data & 0b1111;
    }
}