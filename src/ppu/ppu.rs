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


//todo! maybe get rid of this code or find some way to implement it(in a more optmized way since calling this every single clock of the ppu isnt great)
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum State {
    Visible,
    PostRender,
    VBlank,
    PreRender,
}
#[allow(dead_code)]
impl State {
    fn current_ppu_state(&mut self, scanline: &i16) -> State {
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
    pub ctrl:   PpuCtrlFlags,
    pub mask:   PpuMaskFlags,
    pub status: PpuStatusFlags,

    data_buffer: u8, // leitura de $2007 é atrasada um ciclo

    pub oam: [u8; 0x100],

    // ── Registers Loopy ───────────────────────────────────────────
    pub v: PPUAddress, // endereço VRAM atual
    pub t: PPUAddress, // endereço VRAM temporário (canto superior esquerdo)
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

    // ── Sprites ──────────────────────────────────────────────────
    sprite_scanline:        [(u8, u8, u8, u8); 8], // (y, tile_id, atributes, x)
    sprite_count:           usize,
    sprite_shifter_lo:      [u8; 8],
    sprite_shifter_hi:      [u8; 8],
    sprite0_hit_possible:   bool
}



impl PPU {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        PPU { 
            cycle: 0, 
            scanline: 0, 

            frame_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4].into_boxed_slice(),  // *4 to RGBA channels
            frame_complete: false,

            state:      State::Visible,

            ppubus:     PPUBUS::new(mapper),

            nmi_occurred: false,
            odd_frame: false,

            ctrl: PpuCtrlFlags::new(),
            mask: PpuMaskFlags::new(),
            status: PpuStatusFlags::new(),
            data_buffer: 0,

            oam: [0; 0x100],

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

            sprite_scanline:    [(0,0,0,0); 8],
            sprite_count:           0,
            sprite_shifter_lo:      [0; 8],
            sprite_shifter_hi:      [0; 8],
            sprite0_hit_possible: false
            
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
                let nmi_before = self.ctrl.generate_vblank_nmi();
                
                self.ctrl = PpuCtrlFlags::from_bits_truncate(val);
                
                let nmi_after = self.ctrl.generate_vblank_nmi();
                
                if !nmi_before && nmi_after && self.status.contains(PpuStatusFlags::VblankFlag) {
                    self.nmi_occurred = true;
                }

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
        for _ in 0..cycles {
            self.clock();
        }
        self.frame_complete
    }

    #[inline(always)]
    fn clock(&mut self) {
        match self.scanline {
            -1 | 0..=239 => self.render_scanline(),
            240           => {} // idle
            241           => {
                if self.cycle == 1 {
                    self.status.insert(PpuStatusFlags::VblankFlag);
                    if self.ctrl.generate_vblank_nmi() {
                        self.nmi_occurred = true
                    }
                }
            }
            _             => {} // scanline 241 - 260
        }
        self.increase_cycle();
    }
    #[inline]
    fn increase_cycle(&mut self) {
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 260 {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }
    #[inline]
    fn render_scanline(&mut self) {
        self.render_scanline_background();
        if self.cycle == 257 && self.scanline >= 0 {
            self.render_scanline_sprites();
        }

        let is_prerender = self.scanline == -1;
        let is_visible = self.scanline >= 0;

        if is_prerender && self.cycle == 1 {
            self.status.remove(PpuStatusFlags::VblankFlag);
            self.status.remove(PpuStatusFlags::SpriteOverflow);
            self.status.remove(PpuStatusFlags::Sprite0hit);
        }

        // ── Pre-render: loads Y from t -> v at the cycles: 280-304 ───────
        if is_prerender 
        && self.cycle >= 280 
        && self.cycle <= 304
        && self.mask.contains(PpuMaskFlags::EnableBackground) {
            self.v.transfer_address_y(self.t);
        }

        if is_visible && self.cycle >= 1 && self.cycle <= 256 {
            self.render_pixel();
        }
    }
    #[inline]
    fn render_scanline_sprites(&mut self) {
        self.evaluate_sprites();
        self.load_sprite_shifters();
    }
    #[inline]
    fn evaluate_sprites(&mut self) {
        self.clean_previous_scanline_sprite_registers();

        let mut oam_idx = 0;
        while oam_idx < 64 && self.sprite_count < 8 {
            let sprite_y = self.oam[oam_idx * 4 ] as i16;

            // evaluates if the current scanline is whithin the pos + size of the sprite
            let diff = self.scanline - sprite_y - 1;
            if diff >= 0 && diff < 8 {
                if oam_idx == 0 {
                    self.sprite0_hit_possible = true
                }
                self.sprite_scanline[self.sprite_count] = (
                    sprite_y as u8,
                    self.oam[oam_idx * 4 + 1], // tile id
                    self.oam[oam_idx * 4 + 2], // atributes
                    self.oam[oam_idx * 4 + 3], // x cord
                );
                self.sprite_count += 1
            }
            oam_idx += 1;
        }
    }
    #[inline]
    fn clean_previous_scanline_sprite_registers(&mut self) {
        self.sprite_count = 0;
        self.sprite0_hit_possible = false;
        self.sprite_shifter_lo = [0; 8];
        self.sprite_shifter_hi = [0; 8];
    }
    #[inline]
    fn render_scanline_background(&mut self) { 
        let in_fetch_range = (self.cycle >= 1 && self.cycle <= 256)
        || (self.cycle >= 321 && self.cycle <= 336); // fetch tile range (cycles 1-256 and 321-336)

        if in_fetch_range {
            self.update_shifters();

            match (self.cycle - 1) % 8 {
                0 => {
                    self.load_background_shifters();
                    self.bg_next_tile_id = self.ppubus.read_ppubus(self.v.get_nametable_addr())
                } //reads tiles from nametable (which tile to draw)
                2 => {
                    let attr = self.ppubus.read_ppubus(self.v.get_attribute_addr());
                    let shift = ((self.v.get_coarse_y() & 0b10) << 1) | (self.v.get_coarse_x() & 0b10);
                    self.bg_next_tile_attr = (attr >> shift) & 0b11;
                } //reads atribute (which palete to use)
                4 => {
                    let addr = self.v.get_pattern_table_addr(self.ctrl, self.bg_next_tile_id);
                    self.bg_next_tile_lo = self.ppubus.read_ppubus(addr);
                } //reads low bit plane   (pattern table, plane 0)
                6 => {
                    let addr = self.v.get_pattern_table_addr(self.ctrl, self.bg_next_tile_id) + 8;
                    self.bg_next_tile_hi = self.ppubus.read_ppubus(addr);
                }
                7 => {
                    if self.mask.contains(PpuMaskFlags::EnableBackground) {
                        self.v.increment_coarse_x();
                    }
                } // reads high bit plane (pattern table, bit 1) and increments coarse x
                _ => {}
            }
        }

        // ── end of scanline adjusts ──────────────────────────────────────
        if self.cycle == 256 && self.mask.contains(PpuMaskFlags::EnableBackground) {
            self.v.increment_fine_y();
        }
        if self.cycle == 257 {
            self.load_background_shifters();
            if self.mask.contains(PpuMaskFlags::EnableBackground) {
                //println!("Transfer X: t={:#06x} → v antes={:#06x}", self.t.addr, self.v.addr);
                self.v.transfer_address_x(self.t);
                //println!("  v depois={:#06x} (fine_x={})", self.v.addr, self.fine_x);
            }
        }
    }
    #[inline]
    fn load_background_shifters(&mut self) {
        self.bg_shift_lo = (self.bg_shift_lo & 0xFF00) | self.bg_next_tile_lo as u16;
        self.bg_shift_hi = (self.bg_shift_hi & 0xFF00) | self.bg_next_tile_hi as u16;

        self.bg_attr_shift_lo = (self.bg_attr_shift_lo & 0xFF00)
            | if self.bg_next_tile_attr & 0x01 != 0 {0xFF} else {0x00};
        self.bg_attr_shift_hi = (self.bg_attr_shift_hi & 0xFF00)
            | if self.bg_next_tile_attr & 0x02 != 0 {0xFF} else {0x00};

    }
    #[inline]
    fn load_sprite_shifters(&mut self) {
        let sprite_pattern_base: u16 = if self.ctrl.contains(PpuCtrlFlags::SpritePattern) {0x1000} else {0x0000};

        for i in 0..self.sprite_count {
            let (sprite_y, tile_id, attr, _) = self.sprite_scanline[i];
            let flip_v = attr & 0x80 != 0;

            // usa i16 para evitar overflow
            let row = (self.scanline - sprite_y as i16 - 1) as u8;
            let row = if flip_v { 7 - row } else { row };

            let addr = sprite_pattern_base + tile_id as u16 * 16 + row as u16;
            self.sprite_shifter_lo[i] = self.ppubus.read_ppubus(addr);
            self.sprite_shifter_hi[i] = self.ppubus.read_ppubus(addr + 8);
        }
    }
    #[inline]
    fn update_shifters(&mut self) {
        if self.mask.contains(PpuMaskFlags::EnableBackground) {
            self.bg_attr_shift_hi   <<= 1;
            self.bg_attr_shift_lo   <<= 1;
            self.bg_shift_hi        <<= 1;
            self.bg_shift_lo        <<= 1;
        }
    }
    #[inline]
    fn render_pixel(&mut self) {
        let mux: u16 = 0x8000 >> self.fine_x;

        // ── background ───────────────────────────────────────────────
        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;
        if self.mask.contains(PpuMaskFlags::EnableBackground) {
            let p0 = ((self.bg_shift_lo & mux) != 0) as u8;
            let p1 = ((self.bg_shift_hi & mux) != 0) as u8;
            bg_pixel = (p1 << 1) | p0;

            let a0 = ((self.bg_attr_shift_lo & mux) != 0) as u8;
            let a1 = ((self.bg_attr_shift_hi & mux) != 0) as u8;
            bg_palette = (a1 << 1) | a0;
        }

        // ── sprites ──────────────────────────────────────────────────
        let mut sp_pixel = 0u8;
        let mut sp_palette = 0u8;
        let mut sp_priority = false;
        let mut sp_zero_rendered = false;

        if self.mask.contains(PpuMaskFlags::EnableSprites) {
            let x = (self.cycle - 1) as u8;
            for i in 0..self.sprite_count {
                let sprite_x = self.sprite_scanline[i].3;
                if x < sprite_x || x >= sprite_x.wrapping_add(8) {
                    continue;
                }

                let offset = x - sprite_x;
                let flip_h = self.sprite_scanline[i].2 & 0x40 != 0;
                let bit = if flip_h { offset } else { 7 - offset};

                let lo = (self.sprite_shifter_lo[i] >> bit) & 1;
                let hi = (self.sprite_shifter_hi[i] >> bit) & 1;
                sp_pixel = (hi << 1) | lo;

                if sp_pixel != 0 {
                    sp_palette = (self.sprite_scanline[i].2 & 0x03) + 4;
                    sp_priority = self.sprite_scanline[i].2 & 0x20 == 0;
                    if i == 0 {
                        sp_zero_rendered = true
                    }
                    break;
                }
            }
        }

        // ── sprite 0 hit ─────────────────────────────────────────────
        if self.sprite0_hit_possible && sp_zero_rendered
        /* && bg_pixel != 0 todo! */  && sp_pixel != 0
        && self.cycle >= 2 && self.cycle < 256
        && !self.status.contains(PpuStatusFlags::Sprite0hit) {
            self.status.insert(PpuStatusFlags::Sprite0hit);
        }

        // ── pixel final ──────────────────────────────────────────────
        let (final_pixel, final_palette) = match (bg_pixel, sp_pixel) {
            (0, 0) => (0u8, 0u8),
            (0, _) => (sp_pixel, sp_palette),
            (_, 0) => (bg_pixel, bg_palette),
            _ => if sp_priority {(sp_pixel, sp_palette)} else {(bg_pixel, bg_palette)}
        };

        let palette_addr = if final_pixel == 0 {
            0x3F00
        } else {
            0x3F00 | (final_palette as u16) << 2 | final_pixel as u16
        };

        let color_idx = (self.ppubus.read_ppubus(palette_addr) as usize) & 0x3F;
        let color = NTSC_PALETTE[color_idx];

        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;
        let i = (y * SCREEN_WIDTH + x) * 4;

        self.frame_buffer[i]        = color.r;
        self.frame_buffer[i + 1]    = color.g;
        self.frame_buffer[i + 2]    = color.b;
        self.frame_buffer[i + 3]    = 255; //RGBA alpha always 255
    }

    #[inline]
    pub fn oam_dma_write(&mut self, data: &[u8; 256]) {
        self.oam.copy_from_slice(data);
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