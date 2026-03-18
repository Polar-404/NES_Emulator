use crate::{memory::mappers::Mapper};

use core::panic;
use std::{rc::Rc, vec};
use std::cell::RefCell;

use super::{
    ppuaddr::PPUAddress,
    ppubus::PPUBUS,
    palettes::NTSC_PALETTE,

    registers::{ PpuCtrlFlags, PpuStatusFlags, PpuMaskFlags },
};

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

#[derive(Debug, Clone)]
enum State {
    Visible,
    PostRender,
    VBlank,
    PreRender,
}

impl State {
    fn match_ppu_state(&mut self, scanline: &usize) -> State {
        match scanline {
            0..=239   => Self::Visible,
            240       => Self::PostRender,
            241..=260 => Self::VBlank,
            _         => Self::PreRender
        }
    }
}

pub struct PPU {

    pub cycle: u16,
    pub scanline: i16,

    pub frame_buffer: Box<[u8]>, // *3 to RGB channels
    pub frame_complete: bool,

    pub state: State,

    pub ppubus: PPUBUS,

    pub nmi_occurred: bool,

    odd_frame: bool,

    // ── Registers($2000–$2007) ───────────────────────────────────
    ctrl:   PpuCtrlFlags,
    mask:   PpuMaskFlags,
    status: PpuStatusFlags,

    data_buffer: u8, // leitura de $2007 é atrasada um ciclo

    // ── Registers Loopy ───────────────────────────────────────────
    v: PPUAddress, // endereço VRAM atual
    t: PPUAddress, // endereço VRAM temporário (canto superior esquerdo)
    fine_x: u8,    // scroll X fino (3 bits)
    w: bool,       // latch de escrita: false = primeira, true = segunda

    // ── Background ────────────────────────────────────────────────
    bg_next_tile_id:   u8,
    bg_next_tile_attr: u8,
    bg_next_tile_lo:   u8,
    bg_next_tile_hi:   u8,

    // ── background shift registers (16 bits = 2 tiles) ────────────
    bg_shift_lo:      u16,
    bg_shift_hi:      u16,
    bg_attr_shift_lo: u16,
    bg_attr_shift_hi: u16,
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        PPU { 
            cycle: 0, 
            scanline: 0, 

            frame_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3].into_boxed_slice(), 
            frame_complete: false,

            state:      State::Visible,

            ppubus:     PPUBUS::new(mapper),

            nmi_occurred: false,
            odd_frame: false,

            ctrl: PpuCtrlFlags::new(),
            mask: PpuMaskFlags::new(),
            status: PpuStatusFlags::new(),
            data_buffer: 0,

            v: PPUAddress::new(),
            t: PPUAddress::new(),
            fine_x: 0,   
            w: false,

            bg_next_tile_id:   0,
            bg_next_tile_attr: 0,
            bg_next_tile_lo:   0,
            bg_next_tile_hi:   0,

            bg_shift_lo:      0,
            bg_shift_hi:      0,
            bg_attr_shift_lo: 0,
            bg_attr_shift_hi: 0,
            
        }
    }

    /// https://www.nesdev.org/wiki/PPU_registers#Summary
    pub fn read_registers(&mut self, addr: u8) -> u8 {
        match addr & 0x07 {

            0x02 => {
                let status = (self.status.bits() & 0xE0) | (self.data_buffer & 0x1F);
                self.status.remove(PpuStatusFlags::VblankFlag);
                self.w = false;
                status
            }

            0x04 => {
                //	OAM data read/write
                //todo!()
                0
            }

            0x07 => {
                //reading is delayed by a cycle, it returns the old buffer and loads next one

                let data = self.data_buffer;
                self.data_buffer = self.ppubus.read_ppubus(self.v.addr);

                // EXCEPTION: palette ram has no delay, it discards the old addr and returns the current one
                let result = if self.v.addr >= 0x3F00 {
                    self.data_buffer
                } else {
                    data
                };

                if self.ctrl.contains(PpuCtrlFlags::IncrementVRAM) {
                    self.v.addr = self.v.addr.wrapping_add(32);
                } else {
                    self.v.addr = self.v.addr.wrapping_add(1);
                }

                result
            }

            _ => panic!("Program tried to read register ${:?}", (addr & 0x07)),

        }
    }

    /// https://www.nesdev.org/wiki/PPU_registers#Summary
    pub fn write_registers(&mut self, addr: u16, val: u8) -> bool {
        match addr & 0x07 {
            0x00 => {
                self.ctrl = PpuCtrlFlags::from_bits_truncate(val);
                let base_nametable_address = (val & 0b11) as u16;
                self.t.addr = (self.t.addr & 0b111_00_11111_11111) | (base_nametable_address << 10)
            }
            0x01 => {
                self.mask =PpuMaskFlags::from_bits_truncate(val);
            }
            0x02 => {}
            0x03 => {
                //OAM ADDR 
                //todo!()
            }
            0x04 => {
                //OAM DATA 
                //todo!()
            }
            //https://www.nesdev.org/wiki/PPU_registers#PPUSCROLL
            0x05 => {
                if !self.w {
                    self.fine_x = val & 0b111;
                    self.t.set_coarse_x(val >> 3);
                    self.w = true;
                } else {
                    self.t.set_fine_y(val & 0b111);
                    self.t.set_coarse_y(val >> 3);
                    self.w = false;
                }
            }
            //https://www.nesdev.org/wiki/PPU_registers#PPUADDR
            //The CPU writes to VRAM through a pair of registers on the PPU by first loading
            // an address into PPUADDR and then writing data repeatedly to PPUDATA. The VRAM
            0x06 => {
                if !self.w {
                    self.t.addr = ((val as u16 & 0x3F) << 8) | (self.t.addr & 0x00FF);
                    self.w = true
                } else {
                    self.t.addr = val as u16 | (self.t.addr & 0xFF00);
                    self.v = self.t;
                    self.w = false
                }
            }
            0x07 => {
                self.ppubus.write_ppubus(self.v.addr, val);
                if self.ctrl.contains(PpuCtrlFlags::IncrementVRAM) {
                    self.v.addr = self.v.addr.wrapping_add(32)
                } else {
                    self.v.addr = self.v.addr.wrapping_add(1)
                }
            }

            _ => panic!("unvalid write addr: {}", addr)
        }
        false
    }

    pub fn tick(&mut self, cycles: u16) -> bool {
        todo!()
    }
}

mod write_registers_tests {
    use super::*;
    use crate::memory::dummy_mapper::TestMapper;

    #[allow(unused)]
    fn make_ppu() -> PPU {
        PPU::new(TestMapper::new(vec![]))
    }

    #[test]
    fn write_ppuctrl_updates_nametable_in_t() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x00, 0b00000011); // nametable = 3
        assert_eq!((ppu.t.addr >> 10) & 0x03, 3);
    }

    #[test]
    fn write_ppuscroll_first_write_updates_fine_x_and_coarse_x() {
        let mut ppu = make_ppu();
        // val = 0b00101_011 -> coarse_x = 5, fine_x = 3
        ppu.write_registers(0x05, 0b00101_011);
        assert_eq!(ppu.fine_x, 3);
        assert_eq!(ppu.t.get_coarse_x(), 5);
        assert!(ppu.w);
    }

    #[test]
    fn write_ppuscroll_second_write_updates_coarse_y_and_fine_y() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x05, 0b00101_011); // first
        ppu.write_registers(0x05, 0b01000_110); // second: coarse_y = 8, fine_y = 6
        assert_eq!(ppu.t.get_coarse_y(), 8);
        assert_eq!(ppu.t.get_fine_y(), 6);
        assert!(!ppu.w);
    }

    #[test]
    fn write_ppuaddr_clears_bit_15() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x06, 0xFF);
        ppu.write_registers(0x06, 0x00);
        assert_eq!(ppu.v.addr & 0x8000, 0); // bit 15 must be zero
    }

    #[test]
    fn write_ppuaddr_two_writes_update_v() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x06, 0x21); // high byte
        ppu.write_registers(0x06, 0x00); // low byte -> v = $2100
        assert_eq!(ppu.v.addr, 0x2100);
    }
    #[test]
    fn write_ppudata_increments_v_by_1_by_default() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x06, 0x20);
        ppu.write_registers(0x06, 0x00);
        ppu.write_registers(0x07, 0xAB);
        assert_eq!(ppu.v.addr, 0x2001);
    }

    #[test]
    fn write_ppudata_increments_v_by_32_when_flag_set() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x00, 0b00000100); // IncrementVRAM
        ppu.write_registers(0x06, 0x20);
        ppu.write_registers(0x06, 0x00);
        ppu.write_registers(0x07, 0xAB);
        assert_eq!(ppu.v.addr, 0x2020);
    }
}
mod read_registers_tests {
    use super::*;
    use crate::memory::dummy_mapper::TestMapper;

    #[allow(unused)]
    fn make_ppu() -> PPU {
        PPU::new(TestMapper::new(vec![]))
    }

    #[test]
    fn read_ppustatus_returns_vblank_flag() {
        let mut ppu = make_ppu();
        ppu.status.insert(PpuStatusFlags::VblankFlag);
        let val = ppu.read_registers(0x02);
        assert!(val & 0x80 != 0);
    }

    #[test]
    fn read_ppustatus_clears_vblank_flag() {
        let mut ppu = make_ppu();
        ppu.status.insert(PpuStatusFlags::VblankFlag);
        ppu.read_registers(0x02);
        assert!(!ppu.status.contains(PpuStatusFlags::VblankFlag));
    }

    #[test]
    fn read_ppustatus_resets_write_latch() {
        let mut ppu = make_ppu();
        ppu.w = true;
        ppu.read_registers(0x02);
        assert!(!ppu.w);
    }

    #[test]
    fn read_ppudata_is_buffered() {
        let mut ppu = make_ppu();
        // writes something in VRAM at $2000
        ppu.write_registers(0x06, 0x20);
        ppu.write_registers(0x06, 0x00);
        ppu.write_registers(0x07, 0xAB);

        // resets addr to $2000
        ppu.write_registers(0x06, 0x20);
        ppu.write_registers(0x06, 0x00);

        // first read: returns buffer's trash (no data yet)
        let first = ppu.read_registers(0x07);
        //second read: now returns 0xAB
        let second = ppu.read_registers(0x07);

        assert_ne!(first, 0xAB);
        assert_eq!(second, 0xAB);
    }

    #[test]
    fn read_ppudata_palette_has_no_delay() {
        let mut ppu = make_ppu();

        ppu.write_registers(0x06, 0x3F);
        ppu.write_registers(0x06, 0x05);
        ppu.write_registers(0x07, 0x15);

        ppu.write_registers(0x06, 0x3F);
        ppu.write_registers(0x06, 0x05);
        
        let val = ppu.read_registers(0x07);
        assert_eq!(val, 0x15);
    }

    #[test]
    fn read_ppudata_increments_v_by_1() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x06, 0x20);
        ppu.write_registers(0x06, 0x00);
        ppu.read_registers(0x07);
        assert_eq!(ppu.v.addr, 0x2001);
    }

    #[test]
    fn read_ppudata_increments_v_by_32_when_flag_set() {
        let mut ppu = make_ppu();
        ppu.write_registers(0x00, 0b00000100); // IncrementVRAM
        ppu.write_registers(0x06, 0x20);
        ppu.write_registers(0x06, 0x00);
        ppu.read_registers(0x07);
        assert_eq!(ppu.v.addr, 0x2020);
    }
}