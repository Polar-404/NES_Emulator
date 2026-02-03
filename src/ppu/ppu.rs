//PPUCTRL	$2000	VPHB SINN	            W	    NMI enable (V), PPU master/slave (P), sprite height (H), background tile select (B), sprite tile select (S), increment mode (I), nametable select / X and Y scroll bit 8 (NN)
//PPUMASK	$2001	BGRs bMmG	            W	    color emphasis (BGR), sprite enable (s), background enable (b), sprite left column enable (M), background left column enable (m), greyscale (G)
//PPUSTATUS	$2002	VSO- ----	            R	    vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
//OAMADDR	$2003	AAAA AAAA	            W	    OAM read/write address
//OAMDATA	$2004	DDDD DDDD	            RW	    OAM data read/write
//PPUSCROLL	$2005	XXXX XXXX YYYY YYYY	    Wx2	    X and Y scroll bits 7-0 (two writes: X scroll, then Y scroll)
//PPUADDR	$2006	..AA AAAA AAAA AAAA	    Wx2	    VRAM address (two writes: most significant byte, then least significant byte)
//PPUDATA	$2007	DDDD DDDD	            RW	    VRAM data read/write
//OAMDMA	$4014	AAAA AAAA	            W	    OAM DMA high address

//R  - Readable
//W  - Writeable
//x2 - Internal 2-byte state accessed by two 1-byte accesses


// ------------------- PPU CNTRL FLAGS ------------------- 
//  7  bit  0
//  ---- ----
//  VPHB SINN
//  |||| ||||
//  |||| ||++- Base nametable address
//  |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
//  |||| |+--- VRAM address increment per CPU read/write of PPUDATA
//  |||| |     (0: add 1, going across; 1: add 32, going down)
//  |||| +---- Sprite pattern table address for 8x8 sprites
//  ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
//  |||+------ Background pattern table address (0: $0000; 1: $1000)
//  ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
//  |+-------- PPU master/slave select
//  |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
//  +--------- Vblank NMI enable (0: off, 1: on)

// ------------------- PPU CNTRL FLAGS ------------------- 

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

use crate::{memory::mappers::Mapper};

use std::rc::Rc;
use std::cell::RefCell;

use super::{
    ppuaddr::PPUAddress,
    ppubus::PPUBUS,
    palettes::NTSC_PALETTE,
    
    registers::DoubleWriteRegister,
    registers::PpuCtrlFlags,
    registers::PpuStatusFlags
};

pub struct PPU {
    pub cycle: u16,
    pub scanline: u16,
    pub frame_complete: bool,
    pub frame_buffer: Vec<u8>, // [u8; 256 * 240 * 4], HAS TO ALOCATE ON THE HEAP OTHERWISE IT OVERFLOWS THE STACK

    pub ppu_ctrl:   PpuCtrlFlags,
    pub ppu_mask:   u8,
    pub ppu_status: PpuStatusFlags,
    pub oam_addr:   u8,
    pub oam_data:   u8,
    pub ppu_scrl:   DoubleWriteRegister,

    ///VRAM ADDR
    pub ppu_addr:   DoubleWriteRegister,
    
    pub ppu_data:   u8,
    pub ppu_data_buffer: u8,

    pub oam:    [u8; 0xff], //[0x4014] adress
    pub ppubus:     PPUBUS,


    pub internal_v: PPUAddress, // vram address during the rendering
    pub internal_t: PPUAddress, // temporary vram address (used by PPUSCROLL and PPUADDR)

    pub fine_x: u8,

    //shifters
    pub bg_nametable_byte:          u8,
    pub bg_attribute_byte:          u8,
    pub bg_low_byte:                u8,
    pub bg_high_byte:               u8,
    pub bg_shifter_pattern_low:     u16,
    pub bg_shifter_pattern_high:    u16,
    pub bg_shifter_attribute_low:   u16,
    pub bg_shifter_attribute_high:  u16,

    sprites_in_scanline: u8,
    sprite_0_in_scanline: bool,
    secondary_oam: [u8; 32],
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        PPU {
            cycle: 0,
            scanline: 0,
            frame_complete: false,
            frame_buffer: vec![0; 256 * 240 * 4],

            ppu_ctrl:   PpuCtrlFlags::new(),

            ppu_status: PpuStatusFlags::new(),

            ppu_mask:   0,
            oam_addr:   0,
            oam_data:   0,
            ppu_scrl:   DoubleWriteRegister::new(),
            ppu_addr:   DoubleWriteRegister::new(),
            ppu_data:   0,
            ppu_data_buffer: 0,
            oam: [0; 0xff],

            ppubus:     PPUBUS::new(mapper),

            internal_v: PPUAddress::new(),
            internal_t: PPUAddress::new(),

            fine_x: 0,

            bg_nametable_byte:          0,
            bg_attribute_byte:          0,
            bg_low_byte:                0,
            bg_high_byte:               0,
            bg_shifter_pattern_low:     0,
            bg_shifter_pattern_high:    0,
            bg_shifter_attribute_low:   0,
            bg_shifter_attribute_high:  0,

            sprites_in_scanline: 0,
            sprite_0_in_scanline: false,
            secondary_oam: [0xff; 32],
        }
    }


    pub fn read_registers(&mut self, addr: u8) -> u8 {
        match addr {
            0 => {
                self.ppu_ctrl.bits()
            }
            1 => {
                self.ppu_mask
            }
            2 => {
                let data = self.ppu_status.bits();
                self.ppu_status.remove(PpuStatusFlags::VblankFlag);
                self.ppu_addr.reset_latch();
                self.ppu_scrl.reset_latch();
                data
            }
            3 => {
                self.oam_addr
            }
            4 => {
                self.oam_data
            }
            5 => {
                //cpu cant read from this register
                //TODO remember to make a read_function just for the PPU to read its own BUS
                panic!("Error, CPU tried to read PPU REGISTER 2005, which is for write only")
            }
            6 => {
                panic!("Error, CPU tried to read PPU REGISTER 2006, which is for write only")
            }
            7 => {
                self.ppu_data
            }
            _ => panic!("Error, an invalid address was called: {}", addr)
        }
    }

    pub fn write_registers(&mut self, addr: u16, data: u8)  -> bool {
        match addr {
            0 => { // PPUCTRL
                let before_nmi_status = self.ppu_ctrl.generate_vblank_nmi();
                self.ppu_ctrl = PpuCtrlFlags::from_bits_truncate(data);

                // Check for immediate NMI trigger
                let after_nmi_status = self.ppu_ctrl.generate_vblank_nmi();
                
                //If NMI was just enabled AND the PPU is already in vblank, trigger an NMI
                if !before_nmi_status && after_nmi_status && self.ppu_status.contains(PpuStatusFlags::VblankFlag) {
                    return true
                }
                false
            }
            1 => {
                self.ppu_mask   = data;
                false
            }
            2 => {
                self.ppu_status = PpuStatusFlags::from_bits_truncate(data);
                false
            }
            3 => {
                self.oam_addr   = data;
                false
            }
            4 => {
                self.oam_data   = data;
                false
            }
            5 => { // PPUSCROLL
                if self.ppu_scrl.is_first_write {
                    // Primeira escrita: Coarse X e Fine X
                    self.ppu_scrl.write_byte(data);
                    self.internal_t.set_coarse_x((data >> 3) as u8);

                    self.fine_x = data & 0b0000_0111
                    // Salva o fine X scroll, necessário para a renderização
                    // (você precisa de uma nova variável para isso, por exemplo: `fine_x`)
                } else {

                    self.ppu_scrl.write_byte(data);
                    self.internal_t.set_fine_y(data & 0x07);
                    self.internal_t.set_coarse_y((data >> 3) as u8);
                };
                false
            },
            6 => { // PPUADDR
                if self.ppu_addr.is_first_write {
                    self.ppu_addr.write_byte(data);
                    self.internal_t.addr = (self.internal_t.addr & 0x00FF) | (((data as u16) & 0x3F) << 8);
                } else {
                    self.ppu_addr.write_byte(data);
                    self.internal_t.addr = (self.internal_t.addr & 0xFF00) | (data as u16);
                    self.internal_v.addr = self.internal_t.addr;
                };
                false
            },
            7 => { // PPUDATA ($2007)
                // 1. Escreve o dado na memória usando o endereço 'v' (internal_v)
                self.ppubus.write_ppubus(self.internal_v.addr, data);

                // 2. INCREMENTA o endereço 'v' automaticamente
                // O PPUCTRL diz se deve somar 1 (horizontal) ou 32 (vertical)
                if self.ppu_ctrl.contains(PpuCtrlFlags::IncrementVRAM) {
                    self.internal_v.addr = self.internal_v.addr.wrapping_add(32);
                } else {
                    self.internal_v.addr = self.internal_v.addr.wrapping_add(1);
                }
                
                // Retorna false, pois escrever pixel não gera NMI
                false 
            }
            _ => panic!("um endereço invalido foi chamado: {}", addr)
        }
    }
    
    ///RETURNS IF A NMI (Non-Maskable Interrupt) SHOULD BE ACTIVATED (which starts a VBLANK)
    ///the ppu displays a 256x240 resolution, even though the ppu works at a bigger resolution than that' 
    ///the nes ppu makes 340 cycles per scanline
    ///and 260 scanlines(one for each horizontal line of pixels)
    pub fn tick(&mut self, cycles: u16) -> bool {
        for _ in 0..cycles {
            
            if self.scanline < 240 && self.cycle < 256 {
                //for each 8 cicles, the ppu searchs a new tile:
                if (self.ppu_mask & 0b0000_1000) != 0 {
                    if self.cycle % 8 == 0 {
                        self.swap_tile();
                    }
                }

                //rendering a pixel every cycle
                let bit_mux: u16 = 0x8000 >> self.fine_x;

                // Verifica se o bit está ativo na posição escolhida
                let p0_pixel = (self.bg_shifter_pattern_low & bit_mux) > 0;
                let p1_pixel = (self.bg_shifter_pattern_high & bit_mux) > 0;

                // Combina os bits (transforma true/false em 1/0)
                let pixel_index = ((p1_pixel as u8) << 1) | (p0_pixel as u8);

                // Mesma coisa para a paleta
                let p0_palette = (self.bg_shifter_attribute_low & bit_mux) > 0;
                let p1_palette = (self.bg_shifter_attribute_high & bit_mux) > 0;
                let palette_index = ((p1_palette as u8) << 1) | (p0_palette as u8);

                let color_index_in_palette = self.ppubus.read_ppubus(
                    0x3F00 + (palette_index as u16 * 4) + pixel_index as u16
                );
                let color = NTSC_PALETTE[color_index_in_palette as usize];

                let pixel_index = (self.scanline as usize * 256 + self.cycle as usize) * 4;
                if pixel_index + 3 < self.frame_buffer.len() {

                    self.frame_buffer[pixel_index as usize]     = color.r;
                    self.frame_buffer[pixel_index as usize + 1] = color.g;
                    self.frame_buffer[pixel_index as usize + 2] = color.b;
                    self.frame_buffer[pixel_index as usize + 3] = 255;


                   //CHESS BOARD LOGIC
                   //let color_val = if (self.cycle / 8 + self.scanline / 8) % 2 == 0 { 255 } else { 0 };
                   //self.frame_buffer[pixel_index] = color_val;     // Red
                   //self.frame_buffer[pixel_index + 1] = color_val; // Green
                   //self.frame_buffer[pixel_index + 2] = color_val; // Blue
                   //self.frame_buffer[pixel_index + 3] = 255;       // Alpha
                }

                self.bg_shifter_pattern_high    <<= 1;
                self.bg_shifter_pattern_low     <<= 1;
                self.bg_shifter_attribute_high  <<= 1;
                self.bg_shifter_attribute_low   <<= 1;

            }

            //updating internal v with internal t
            if (self.ppu_mask & 0b0001_1000) != 0 {
                if self.scanline == 261 {
                    //ppu copies repeteadly the vertical t into vertical v between cycles 280 and 304
                    if self.cycle >= 280 && self.cycle <= 304 {
                        self.internal_v.transfer_address_y(self.internal_t);
                    }
                }
                if self.cycle == 257 { // (visible scanlines + pre-render)
                    self.internal_v.transfer_address_x(self.internal_t);
                }

                if self.cycle == 256 {
                    self.internal_v.increment_fine_y();
                }
            }



            self.cycle += 1;

            if self.cycle > 340 {
                self.cycle = 0;
                self.scanline += 1;

                if self.scanline == 241 { // Começa o Vblank
                    self.ppu_status.insert(PpuStatusFlags::VblankFlag);
                    self.frame_complete = true;

                    if self.ppu_ctrl.contains(PpuCtrlFlags::VblankNMI) {
                        return true
                    }
                }
                
                if self.scanline > 261 {
                    self.scanline = 0;
                    self.ppu_status.remove(PpuStatusFlags::VblankFlag);
                    self.ppu_status.remove(PpuStatusFlags::Sprite0hit);
                    self.ppu_status.remove(PpuStatusFlags::SpriteOverflow);

                    self.frame_complete = false;
                }
            }
        }
        false
    }

    fn get_sprites(&mut self) {
        if (257..=320).contains(&self.cycle) {
            self.sprites_in_scanline = 0;
            self.sprite_0_in_scanline = false;
            self.secondary_oam.fill(0xff);
        }
    }

    fn sprite_evaluation(&mut self) {
        let mut sprite_count = 0;
        let sprite_height = if self.ppu_ctrl.contains(PpuCtrlFlags::SpriteSize) { 16 } else { 8 };
        let sprite_0_index: isize = -1;

        for n in 0..64 {
            let y_coord = self.oam[n * 4];
            
            if self.scanline >= y_coord as u16 && self.scanline < (y_coord as u16 + sprite_height) {
                if sprite_count < 8 {
                    self.secondary_oam[sprite_count as usize * 4] = self.oam[n * 4];
                    self.secondary_oam[sprite_count as usize * 4 + 1] = self.oam[n * 4 + 1];
                    self.secondary_oam[sprite_count as usize * 4 + 2] = self.oam[n * 4 + 2];
                    self.secondary_oam[sprite_count as usize * 4 + 3] = self.oam[n * 4 + 3];

                    if n == 0 {
                        self.sprite_0_in_scanline = true;
                    }
                    sprite_count += 1;
                } else {
                    // Set sprite overflow flag
                    self.ppu_status.insert(PpuStatusFlags::SpriteOverflow);
                    break;
                }
            }
        }
    }

    fn swap_tile(&mut self) {
        self.bg_shifter_pattern_high    <<= 8;
        self.bg_shifter_pattern_low     <<= 8;
        self.bg_shifter_attribute_high  <<= 8;
        self.bg_shifter_attribute_low   <<= 8;


        //searching the nametable ID (or TIle ID)
        self.bg_nametable_byte = self.ppubus.read_ppubus(self.internal_v.get_nametable_addr());

        //searching the atribute byte
        self.bg_attribute_byte = self.ppubus.read_ppubus(self.internal_v.get_attribute_addr());

        //takes the pattern low and high byte
        let pattern_addr = self.internal_v.get_pattern_table_addr(
            //self.ppu_ctrl.clone(), //TODO
            self.ppu_ctrl,
            self.bg_nametable_byte
        );
        println!("{:#X}", pattern_addr);
        self.bg_low_byte = self.ppubus.read_ppubus(pattern_addr);
        self.bg_high_byte = self.ppubus.read_ppubus(pattern_addr + 8);

        if self.bg_low_byte != 0 || self.bg_high_byte != 0{
            println!("Li um tile: {:X} / {:X}", self.bg_low_byte, self.bg_high_byte)
        }

        //puts the data back into the shifters
        self.bg_shifter_pattern_low  |= self.bg_low_byte  as u16;
        self.bg_shifter_pattern_high |= self.bg_high_byte as u16;

        // repeat the atribute byte 8 times for each pixel bit
        let attribute_shift = (self.internal_v.get_coarse_y() & 0x02) * 2 + (self.internal_v.get_coarse_x() & 0x02);
        let attribute_palette = (self.bg_attribute_byte >> attribute_shift) & 0x03;

        if (attribute_palette & 0b01) != 0 {
            self.bg_shifter_attribute_low |= 0xFF00;
        }
        if (attribute_palette & 0b10) != 0 {
            self.bg_shifter_attribute_high |= 0xFF00;
        }
        
        //incrmeent sight for the next horizontal tile
        self.internal_v.increment_coarse_x();
    }
    
    //VPHB SINN
    pub fn format_ppu_status(&self, status: u8) -> String {
        let mut s = String::new();
        s.push_str(if (status & 0b10000000) != 0 { "N" } else { "-" });
        s.push_str(if (status & 0b01000000) != 0 { "N" } else { "-" });
        s.push_str(if (status & 0b00100000) != 0 { "I" } else { "-" });
        s.push_str(if (status & 0b00010000) != 0 { "S" } else { "-" });
        s.push_str(if (status & 0b00001000) != 0 { "B" } else { "-" });
        s.push_str(if (status & 0b00000100) != 0 { "H" } else { "-" });
        s.push_str(if (status & 0b00000010) != 0 { "P" } else { "-" });
        s.push_str(if (status & 0b00000001) != 0 { "V" } else { "-" });
        s
    }
}

#[cfg(test)]
mod ppu_test {
    use crate::memory::dummy_mapper::TestMapper;

    use super::*;

    #[test]
    fn test_vblank_flag_timing() {
        let mapper = TestMapper::new(vec![]);
        let mut ppu = PPU::new(mapper);

        //runs all the way to a vblank
        for _ in 0..241 {
            ppu.tick(341); 
        }

        assert!(ppu.ppu_status.contains(PpuStatusFlags::VblankFlag));
        assert!(ppu.frame_complete);
        
        for _ in 0..(262 - 241) {
            ppu.tick(341);
        }
        
        //verifies if it was cleaned
        assert!(!ppu.ppu_status.contains(PpuStatusFlags::VblankFlag));
        assert!(!ppu.frame_complete);
    }
}

//2 registers are responsible for accessing PPU memory map:
//
//Address (0x2006) & Data (0x2007) - provide access to the memory map available for PPU
//3 registers control internal memory(OAM) that keeps the state of sprites
//
//OAM Address (0x2003) & OAM Data (0x2004) - Object Attribute Memory - the space responsible for sprites
//Direct Memory Access (0x4014) - for fast copying of 256 bytes from CPU RAM to OAM
//3 Write-only registers are controlling PPU actions:
//
//Controller (0x2000) - instructs PPU on general logic flow (which memory table to use, if PPU should interrupt CPU, etc.)
//Mask (0x2001) - instructs PPU how to render sprites and background
//Scroll (0x2005) - instructs PPU how to set a viewport