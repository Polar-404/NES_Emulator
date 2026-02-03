bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PpuCtrlFlags: u8 {
        const NameTable1              = 0b00000001; //Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const NameTable2              = 0b00000010; //Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const IncrementVRAM           = 0b00000100; //VRAM address increment per CPU read/write of PPUDATA (0: add 1, going across; 1: add 32, going down)
        const SpritePattern           = 0b00001000; //Sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; ignored in 8x16 mode)
        const BackGroundPattern       = 0b00010000; //Background pattern table address (0: $0000; 1: $1000) Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
        const SpriteSize              = 0b00100000; //Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
        const PPUMasterSlaveSelect    = 0b01000000; //PPU master/slave select
        const VblankNMI               = 0b10000000; //Vblank NMI enable (0: off, 1: on)
    }
}
impl PpuCtrlFlags {
    pub fn new() -> Self {
        PpuCtrlFlags::from_bits_truncate(0b0000_0000)
    }
    pub fn generate_vblank_nmi(&self) -> bool {
        return self.contains(PpuCtrlFlags::VblankNMI);
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct PpuStatusFlags: u8 {
        const X1                = 0b00000001; //(PPU open bus or 2C05 PPU identifier)
        const X2                = 0b00000010; //(PPU open bus or 2C05 PPU identifier)
        const X3                = 0b00000100; //(PPU open bus or 2C05 PPU identifier)
        const X4                = 0b00001000; //(PPU open bus or 2C05 PPU identifier)
        const X5                = 0b00010000; //(PPU open bus or 2C05 PPU identifier)
        const SpriteOverflow    = 0b00100000; //Sprite overflow flag
        const Sprite0hit        = 0b01000000; //Sprite 0 hit flag
        const VblankFlag        = 0b10000000; //Vblank flag, cleared on read. Unreliable;
    }
}
impl PpuStatusFlags {
    pub fn new() -> Self {
        PpuStatusFlags::from_bits_truncate(0b0000_0000)
    }
}

pub struct DoubleWriteRegister {
    pub value: u16,
    pub is_first_write: bool,
}
impl DoubleWriteRegister {
    pub fn new() -> Self {
        DoubleWriteRegister {
            value: 0,
            is_first_write: true,
        }
    }
    
    // Método para a PPU escrever nele
    pub fn write_byte(&mut self, data: u8) {
        if self.is_first_write {
            self.value = ((data as u16) << 8) | (self.value & 0x00FF);
        } else {
            self.value = (self.value & 0xFF00) | (data as u16);
        }
        self.is_first_write = !self.is_first_write;
    }
    
    // Método para resetar o estado de escrita dupla (chamado pela leitura de $2002)
    pub fn reset_latch(&mut self) {
        self.is_first_write = true;
    }
}