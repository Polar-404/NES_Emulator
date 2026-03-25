const NOISE_TIMER_PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
];

#[derive(Debug, Default)]
pub struct Noise {
    pub enabled: bool,
    timer_reload: u16,
    timer_value: u16,

    noise_mode: bool,

    halt: bool,
    length_counter: u8,

    use_constant_vol: bool,
    constant_vol: u8,
    volume_envelope: u8,

    lfsr: u16,

    // envelope
    envelope_start:     bool,
    envelope_divider:   u8,
    envelope_volume:    u8,
    
}
impl Noise {
    pub fn new() -> Self {
        Noise {
            noise_mode: true,
            lfsr: 0x01,
            ..Default::default()
        }
    }

    pub fn step(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_reload;

            let shift_amout = if self.noise_mode { 6 } else { 1 };

            let bit_0 = self.lfsr & 1;
            let other_bit  = (self.lfsr >> shift_amout) & 1;
            let feedback = other_bit ^ bit_0;

            self.lfsr >>= 1;

            self.lfsr |= feedback << 14;
        } else {
            self.timer_value -= 1;
        }
    }

    pub fn get_amplitude(&mut self) -> f32 {
        if !self.enabled || self.length_counter == 0 || self.lfsr & 0x01 == 1 { return 0.0 }

        let vol = if self.use_constant_vol {
            self.constant_vol
        } else {
            self.volume_envelope
        };

        vol as f32
    }

    fn reset_start_envelope(&mut self) {
        self.envelope_start = false;
        self.envelope_volume = 15;
        self.envelope_divider = self.constant_vol;
    }

    fn reset_divider_envelope(&mut self) {
        self.envelope_divider = self.constant_vol;
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
    
    ///$400C
    pub fn write_halt_and_volume(&mut self, data: u8) {
        self.halt = (data >> 5) & 0x01 != 0;
        self.use_constant_vol = (data >> 4) & 0x01 != 0;
        self.volume_envelope = data & 0x0F;
    }

    ///$400E
    pub fn write_noise(&mut self, data: u8) {
        self.noise_mode = data & 0x80 != 0;
        self.timer_reload = NOISE_TIMER_PERIODS[(data & 0x0F) as usize]; 
    }

    ///$400F
    pub fn write_length_counter(&mut self, data: u8) {
        self.length_counter = super::square::LENGTH_TABLE[(data >> 3) as usize];
    }
}