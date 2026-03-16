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
    
    #[inline]
    pub fn generate_vblank_nmi(&self) -> bool {
        self.contains(PpuCtrlFlags::VblankNMI)
    }
}


// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: greyscale)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Enable background rendering
// |||+------ 1: Enable sprite rendering
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue
bitflags! {
    #[derive(Debug)]
    pub struct PpuMaskFlags: u8 {
        const Greyscale         = 0b0000_0001;
        const ShowBackground    = 0b0000_0010;
        const ShowSprites       = 0b0000_0100;
        const EnableBackground  = 0b0000_1000;
        const EnableSprites     = 0b0001_0000;
        const EmphasizeRed      = 0b0010_0000;
        const EmphasizeGreen    = 0b0100_0000;
        const EmphasizeBlue     = 0b1000_0000;
    }
}
impl PpuMaskFlags {
    pub fn new() -> Self {
        PpuMaskFlags::from_bits_truncate(0b0000_0000)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_vblank_nmi_disabled_by_default() {
        let ctrl = PpuCtrlFlags::new();
        assert!(!ctrl.generate_vblank_nmi());
    }

    #[test]
    fn ctrl_vblank_nmi_enabled() {
        let ctrl = PpuCtrlFlags::from_bits_truncate(0b10000000);
        assert!(ctrl.generate_vblank_nmi());
    }

    #[test]
    fn ctrl_background_pattern_bit() {
        let ctrl = PpuCtrlFlags::from_bits_truncate(0b00010000);
        assert!(ctrl.contains(PpuCtrlFlags::BackGroundPattern));
    }

    #[test]
    fn ctrl_increment_vram_bit() {
        let ctrl = PpuCtrlFlags::from_bits_truncate(0b00000100);
        assert!(ctrl.contains(PpuCtrlFlags::IncrementVRAM));
    }

    #[test]
    fn status_vblank_set_and_clear() {
        let mut status = PpuStatusFlags::new();
        assert!(!status.contains(PpuStatusFlags::VblankFlag));
        status.insert(PpuStatusFlags::VblankFlag);
        assert!(status.contains(PpuStatusFlags::VblankFlag));
        status.remove(PpuStatusFlags::VblankFlag);
        assert!(!status.contains(PpuStatusFlags::VblankFlag));
    }
}