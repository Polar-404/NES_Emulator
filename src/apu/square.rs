
const SEQUENCE: [[bool; 8]; 4] = [
    [false, true, false, false, false, false, false, false], // 12.5%
    [false, true, true, false, false, false, false, false],  // 25%
    [false, true, true, true, true, false, false, false],   // 50%
    [true, false, false, true, true, true, true, true],     // 75% (25% invertido)
];

pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Default, Debug)]
pub struct SquareWave {
    pub enabled:    bool,
    pub is_pulse1: bool,
    timer_reload:   u16,
    timer_value:    u16,
    duty_cycle:     u8,
    duty_value:     u8,

    // length counter
    length_counter:     u8,
    halt:               bool, // bit 5: length_halt and envelope_loop

    // envelope
    envelope_start:     bool,
    envelope_divider:   u8,
    envelope_volume:    u8,
    use_constant_vol:   bool,
    constant_volume:    u8,

    // sweep
    sweep_enabled: bool,
    sweep_divider_period: u8,
    sweep_divider: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
}

impl SquareWave {
    pub fn new(is_pulse1: bool) -> Self {
        SquareWave {
            is_pulse1,
            ..Default::default()
        }
    }
    pub fn step(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_reload + 1;
            self.duty_value = (self.duty_value + 1) % 8;
        } else {
            self.timer_value -= 1;
        }
    }
    pub fn get_amplitude(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 || self.is_muted() { return 0.0 }
        if !SEQUENCE[self.duty_cycle as usize][self.duty_value as usize] { return 0.0 };

        let vol = if self.use_constant_vol {
            self.constant_volume
        } else {
            self.envelope_volume
        };

        vol as f32
    }

    fn reset_start_envelope(&mut self) {
        self.envelope_start = false;
        self.envelope_volume = 15;
        self.envelope_divider = self.constant_volume;
    }

    fn reset_divider_envelope(&mut self) {
        self.envelope_divider = self.constant_volume;
        if self.envelope_volume == 0 {
            if self.halt {
                self.envelope_volume = 15;
            }
        } else {
            self.envelope_volume -= 1;
        }
    }

    pub fn clock_length(&mut self) {
        if !self.halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.reset_start_envelope();
        } else if self.envelope_divider == 0 {
            self.reset_divider_envelope();
        } else {
            self.envelope_divider -= 1;
        }
    }
    pub fn write_timer_lo(&mut self, data: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | data as u16
    } 
    pub fn write_timer_hi(&mut self, data: u8) {
        self.timer_reload = (self.timer_reload & 0x00FF) | ((data as u16 & 0b111) << 8);
        self.duty_value = 0;
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        self.envelope_start = true;
    }
    pub fn write_control(&mut self, data: u8) {
        self.duty_cycle       = data >> 6 & 0b11;
        self.constant_volume  = data & 0x0F;
        self.halt             = data & 0x20 != 0;
        self.use_constant_vol = data & 0x10 != 0;
    }
    pub fn write_sweep(&mut self, data: u8) {
        self.sweep_enabled = (data & 0x80) != 0;
        self.sweep_divider_period = (data >> 4) & 0x07;
        self.sweep_negate = (data & 0x08) != 0;
        self.sweep_shift = data & 0x07;
        self.sweep_reload = true;
    }
    ///pitch band
    fn target_period(&self) -> u16 {
        let change = self.timer_reload >> self.sweep_shift;
        if self.sweep_negate {
            if self.is_pulse1 {
                self.timer_reload.saturating_sub(change).saturating_sub(1) // O infame bug do Pulse 1
            } else {
                self.timer_reload.saturating_sub(change)
            }
        } else {
            self.timer_reload + change
        }
    }
    fn is_muted(&self) -> bool {
        self.timer_reload < 8 || self.target_period() > 0x7FF
    }
    ///sweep motor, runs at 120hz
    pub fn clock_sweep(&mut self) {
        if self.sweep_divider == 0 && self.sweep_enabled && !self.is_muted() && self.sweep_shift > 0 {
            self.timer_reload = self.target_period();
        }

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_divider_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }
}