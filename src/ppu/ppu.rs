use crate::{memory::mappers::Mapper};

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

    pub fn read_registers(&mut self, addr: u8) -> u8 {
        todo!()
    }
    pub fn write_registers(&mut self, addr: u16, val: u8) -> bool {
        todo!()
    }
    pub fn tick(&mut self, cycles: u16) -> bool {
        todo!()
    }
}

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