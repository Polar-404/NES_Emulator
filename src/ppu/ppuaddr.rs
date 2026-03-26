use super::registers::PpuCtrlFlags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PPUAddress {
    // yyy NN YYYYY XXXXX
    // ||| || ||||| +++++-- Coarse X scroll
    // ||| || +++++-------- Coarse Y scroll
    // ||| ++-------------- Nametable select
    // +++----------------- Fine Y scroll
    pub addr: u16
}

impl PPUAddress {
    pub fn new() -> Self {
        PPUAddress { addr: 0 }
    }

    #[inline(always)]
    pub fn set_coarse_x(&mut self, data: u8) {
        self.addr = (self.addr & 0b111_11_11111_00000) | (data as u16 & 0x1F)
    }
    #[inline(always)]
    pub fn get_coarse_x(&self) -> u8 {
        (self.addr & 0b11111) as u8
    }

    #[inline]
    pub fn set_coarse_y(&mut self, data: u8) {
        self.addr = (self.addr & 0b111_11_00000_11111) | ((data as u16 & 0x1F) << 5)
    }
    #[inline]
    pub fn get_coarse_y(&self) -> u8 {
        ((self.addr & 0b11111_00000) >> 5) as u8
    }
    #[inline]
    pub fn set_fine_y(&mut self, val: u8) {
        self.addr = (self.addr & 0b000_11_11111_11111) | ((val as u16 & 0x07) << 12)
    }
    #[inline(always)]
    pub fn get_fine_y(&self) -> u8 {
        ((self.addr & 0b111_00_00000_00000) >> 12) as u8
    }
    #[inline]
    pub fn increment_coarse_x(&mut self) {
        let mut coarse_x = self.get_coarse_x();
        if coarse_x == 31 {
            coarse_x = 0;
            self.addr ^= 0x0400;
        } else {
            coarse_x += 1;
        }
        self.set_coarse_x(coarse_x);
    }
    #[inline]
    pub fn increment_fine_y(&mut self) {
        let fine_y = self.get_fine_y();
        if fine_y < 7 {
            self.set_fine_y(fine_y + 1);
        } else {
            self.set_fine_y(0);
            let mut coarse_y = self.get_coarse_y();
            match coarse_y {
                29 => {
                    coarse_y = 0;
                    self.addr ^= 0x0800
                }

                31 => coarse_y = 0,

                _ => coarse_y += 1
            }
            self.set_coarse_y(coarse_y);
        }
    }

    #[inline]
    pub fn get_nametable_addr(&self) -> u16 {
        0x2000 | (self.addr & 0x0FFF)
    }
    #[inline]
    pub fn get_attribute_addr(&self) -> u16 {
        0x23C0 | (self.addr & 0x0C00) | ((self.addr >> 4) & 0x38) | ((self.addr >> 2) & 0x07)
    }

    #[inline]
    pub fn get_pattern_table_addr(&self, ctrl_flags: PpuCtrlFlags, tile_id: u8) -> u16{ 
        let pattern_table_base = if ctrl_flags.contains(PpuCtrlFlags::BackGroundPattern) { 0x1000 } else { 0x0000 };
        let addr = pattern_table_base + (tile_id  as u16 * 16) + self.get_fine_y() as u16;
        addr
    }

    //copies the horizontal position "t" to the current "v"
    #[inline(always)]
    pub fn transfer_address_x(&mut self, source: PPUAddress) {
        const MASK: u16 = 0b000_01_00000_11111; // 0x041F
        self.addr = (self.addr & !MASK) | (source.addr & MASK);
    }
    #[inline(always)]
    pub fn transfer_address_y(&mut self, source: PPUAddress) {
        const MASK: u16 = 0b111_10_11111_00000; // 0x7BE0
        self.addr = (self.addr & !MASK) | (source.addr & MASK);
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    // ── set/get coarse_x ─────────────────────────────────────────────────

    #[test]
    fn coarse_x_roundtrip() {
        let mut a = PPUAddress::new();
        for x in 0u8..=31 {
            a.set_coarse_x(x);
            assert_eq!(a.get_coarse_x(), x);
        }
    }

    #[test]
    fn coarse_x_doesnt_corrupt_other_bits() {
        let mut a = PPUAddress::new();
        a.set_fine_y(0b111);
        a.set_coarse_y(0b11111);
        a.set_coarse_x(0b10101);
        assert_eq!(a.get_fine_y(),   0b111);
        assert_eq!(a.get_coarse_y(), 0b11111);
        assert_eq!(a.get_coarse_x(), 0b10101);
    }

    // ── set/get coarse_y ─────────────────────────────────────────────────

    #[test]
    fn coarse_y_roundtrip() {
        let mut a = PPUAddress::new();
        for y in 0u8..=31 {
            a.set_coarse_y(y);
            assert_eq!(a.get_coarse_y(), y);
        }
    }

    #[test]
    fn coarse_y_doesnt_corrupt_other_bits() {
        let mut a = PPUAddress::new();
        a.set_fine_y(0b111);
        a.set_coarse_x(0b11111);
        a.set_coarse_y(0b10101);
        assert_eq!(a.get_fine_y(),   0b111);
        assert_eq!(a.get_coarse_x(), 0b11111);
        assert_eq!(a.get_coarse_y(), 0b10101);
    }

    // ── set/get fine_y ───────────────────────────────────────────────────

    #[test]
    fn fine_y_roundtrip() {
        let mut a = PPUAddress::new();
        for y in 0u8..=7 {
            a.set_fine_y(y);
            assert_eq!(a.get_fine_y(), y);
        }
    }

    #[test]
    fn fine_y_doesnt_corrupt_other_bits() {
        let mut a = PPUAddress::new();
        a.set_coarse_x(0b11111);
        a.set_coarse_y(0b11111);
        a.set_fine_y(0b101);
        assert_eq!(a.get_coarse_x(), 0b11111);
        assert_eq!(a.get_coarse_y(), 0b11111);
        assert_eq!(a.get_fine_y(),   0b101);
    }

    #[test]
    fn increment_coarse_x_basic() {
        let mut a = PPUAddress::new();
        a.set_coarse_x(0);
        a.increment_coarse_x();
        assert_eq!(a.get_coarse_x(), 1);
    }

    #[test]
    fn increment_coarse_x_wrap_31_to_0_and_changes_nametable_to_h() {
        let mut a = PPUAddress::new();
        a.set_coarse_x(31);
        let nt_before = (a.addr >> 10) & 0x01; // bit 10 = nametable horizontal
        a.increment_coarse_x();
        assert_eq!(a.get_coarse_x(), 0);
        let nt_after = (a.addr >> 10) & 0x01;
        assert_ne!(nt_before, nt_after); // bit deve ter flipado
    }

    #[test]
    fn increment_coarse_x_doesnt_corrupts_other_bits() {
        let mut a = PPUAddress::new();
        a.set_fine_y(0b101);
        a.set_coarse_y(0b10101);
        a.set_coarse_x(5);
        a.increment_coarse_x();
        assert_eq!(a.get_fine_y(),   0b101);
        assert_eq!(a.get_coarse_y(), 0b10101);
        assert_eq!(a.get_coarse_x(), 6);
    }

    // ── increment_fine_y ─────────────────────────────────────────────────

    #[test]
    fn increment_fine_y_basic() {
        let mut a = PPUAddress::new();
        a.set_fine_y(3);
        a.increment_fine_y();
        assert_eq!(a.get_fine_y(), 4);
    }

    #[test]
    fn increment_fine_y_wrap_7_increment_coarse_y() {
        let mut a = PPUAddress::new();
        a.set_fine_y(7);
        a.set_coarse_y(10);
        a.increment_fine_y();
        assert_eq!(a.get_fine_y(),   0);
        assert_eq!(a.get_coarse_y(), 11);
    }

    #[test]
    fn increment_fine_y_coarse_y_29_wrap_and_changes_nametable_to_v() {
        let mut a = PPUAddress::new();
        a.set_fine_y(7);
        a.set_coarse_y(29);
        let nt_before = (a.addr >> 11) & 0x01; // bit 11 = nametable vertical
        a.increment_fine_y();
        assert_eq!(a.get_fine_y(),   0);
        assert_eq!(a.get_coarse_y(), 0);
        let nt_after = (a.addr >> 11) & 0x01;
        assert_ne!(nt_before, nt_after);
    }

    #[test]
    fn increment_fine_y_coarse_y_31_wrap_without_nametable_changes() {
        let mut a = PPUAddress::new();
        a.set_fine_y(7);
        a.set_coarse_y(31);
        let nt_before = (a.addr >> 10) & 0x03;
        a.increment_fine_y();
        assert_eq!(a.get_coarse_y(), 0);
        assert_eq!((a.addr >> 10) & 0x03, nt_before); // nametable intacto
    }
}