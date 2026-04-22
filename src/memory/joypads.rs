bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct JoyPadButtons: u8 {
        const A         = 0b0000_0001;
        const B         = 0b0000_0010;
        const SELECT    = 0b0000_0100;
        const START     = 0b0000_1000;
        const UP        = 0b0001_0000;
        const DOWN      = 0b0010_0000;
        const LEFT      = 0b0100_0000;
        const RIGHT     = 0b1000_0000;
    }
}
pub struct JoyPad {
    read_counter: u8,
    joypad_buttons: JoyPadButtons,
}

impl JoyPad {
    pub fn new() -> Self {
        JoyPad{
            read_counter: 0,
            joypad_buttons: JoyPadButtons::from_bits_truncate(0),
        }
    }
    pub fn read(&mut self) -> u8 {
        if self.read_counter >= 8 {
            return 1
        }
        let bit = (self.joypad_buttons.bits() >> self.read_counter) & 1;
        self.read_counter += 1;
        bit
    }
    pub fn write(&mut self, val: u8) {
        if val & 1 != 0 {
            self.read_counter = 0;
        }
    }
    pub fn set_button(&mut self, button: JoyPadButtons, pressed: bool) {
        if pressed {
            self.joypad_buttons.insert(button);
        } else {
            self.joypad_buttons.remove(button);
        }
    }
}