use crate::{cpu::cpu::CpuFlags, ppu::ppu::PpuCtrlFlags};

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

    #[inline]
    pub fn set_coarse_x(&mut self, data: u8) {
        self.addr = (self.addr & 0b111_11_11111_00000) | data as u16
    }
    #[inline]
    pub fn get_coarse_x(&self) -> u8 {
        (self.addr & 0b11111) as u8
    }

    #[inline]
    pub fn set_coarse_y(&mut self, data: u8) {
        self.addr = (self.addr & 0b111_11_00000_11111) | ((data as u16) << 5)
    }
    #[inline]
    pub fn get_coarse_y(&self) -> u8 {
        (self.addr & 0b11111_00000) as u8
    }

    #[inline]
    pub fn set_nametable(&mut self, val: u8) {
        self.addr = (self.addr & 0b111_00_11111_11111) | ((val as u16) << 10);
    }
    #[inline]
    pub fn get_namtable(&self) -> u8 {
        ((self.addr & 0b000_11_00000_00000) >> 10) as u8
    }

    #[inline]
    pub fn set_fine_y(&mut self, val: u8) {
        self.addr = (self.addr & 0b000_11_11111_11111) | ((val as u16) << 12);
    }
    #[inline]
    pub fn get_fine_y(&self) -> u8 {
        ((self.addr & 0b111_00_00000_00000) >> 12) as u8
    }

    pub fn incrmeent_course_x(&mut self) {
        let mut course_x = self.get_coarse_x();
        if course_x == 31 {
            course_x = 0;
            self.addr ^= 0x0400;
        } else {
            course_x += 1;
        }
        self.set_coarse_x(course_x);
    }
    pub fn incrment_fine_y(&mut self) {
        let fine_y = self.get_coarse_y();
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

    
}