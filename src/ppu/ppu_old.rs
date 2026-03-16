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

// ------------------- PPU MASK FLAGS ------------------- 

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
    pub frame_counter: i64,
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

    pub oam:    [u8; 0x100], //[0x4014] adress
    pub ppubus:     PPUBUS,


    pub internal_v: PPUAddress, // vram address during the rendering
    pub internal_t: PPUAddress, // temporary vram address (used by PPUSCROLL and PPUADDR)

    pub fine_x: u8,

    //shifters

    //background
    pub bg_nametable_byte:          u8,
    pub bg_attribute_byte:          u8,
    pub bg_low_byte:                u8,
    pub bg_high_byte:               u8,
    pub bg_shifter_pattern_low:     u16,
    pub bg_shifter_pattern_high:    u16,
    pub bg_shifter_attribute_low:   u16,
    pub bg_shifter_attribute_high:  u16,

    //sprites
    pub sprite_shifter_pattern_low: [u8; 8],
    pub sprite_shifter_pattern_high: [u8; 8],
    pub sprite_positions: [u8; 8],
    pub sprite_attributes: [u8; 8],
    pub sprite_0_hit_possible: bool,

    pub sprite_count: u8,

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
            frame_counter: 0,
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
            oam: [0; 0x100],

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

            sprite_shifter_pattern_low: [0; 8],
            sprite_shifter_pattern_high: [0; 8],
            sprite_positions: [0xFF; 8],
            sprite_attributes: [0; 8],
            sprite_0_hit_possible: false,

            sprite_count: 0,

            sprites_in_scanline: 0,
            sprite_0_in_scanline: false,
            secondary_oam: [0xff; 32],
        }
    }

    pub fn load_sprite_shifters(&mut self) {
        let sprite_pattern_addr = if self.ppu_ctrl.contains(PpuCtrlFlags::SpritePattern) {0x1000} else {0x00};
        let sprite_height = if self.ppu_ctrl.contains(PpuCtrlFlags::SpriteSize) {16} else {8};

        for i in 0..8 {
            let y_coord     =   self.secondary_oam[i * 4];
            let tile_id     =   self.secondary_oam[i * 4 + 1];
            let attribute   =   self.secondary_oam[i * 4 + 2];
            let x_cord      =   self.secondary_oam[i * 4 + 3];

            self.sprite_attributes[i] = attribute;
            self.sprite_positions [i] = x_cord;

            if y_coord == 0xFF {
                self.sprite_shifter_pattern_low[i] = 0;
                self.sprite_shifter_pattern_high[i] = 0;
                continue;
            }

            let diff_y = self.scanline as i16 - y_coord as i16;

            let row = if (attribute & 0x80) != 0 {
                ((sprite_height as i32 - 1) - diff_y as i32) as u16
            } else {
                diff_y as u16
            };

            let addr = sprite_pattern_addr + (tile_id as u16 * 16) + row;

            let mut pattern_low = self.ppubus.read_ppubus(addr);
            let mut pattern_high = self.ppubus.read_ppubus(addr + 8);

            if (attribute & 0x40) != 0 {
                pattern_low = pattern_low.reverse_bits();
                pattern_high = pattern_high.reverse_bits();
            }

            self.sprite_shifter_pattern_low[i] = pattern_low;
            self.sprite_shifter_pattern_high[i] = pattern_high;

        }

    }

    ///tuple returns (pixel, palette, priority, is_sprite_0)
    fn get_sprite_pixel_x(&self, x: u16) -> (u8, u8, u8, bool) {
        if (self.ppu_mask & 0b0001_0000) == 0 {
            return (0, 0, 0, false);
        }
        for i in 0..8 {
            let sprite_x = self.sprite_positions[i] as u16;

            if x >= sprite_x && x < sprite_x + 8 {
                let offset = x - sprite_x;
                let bit_mux = 0x80 >> offset;

                let p_low = (self.sprite_shifter_pattern_low[i] & bit_mux) > 0;
                let p_high = (self.sprite_shifter_pattern_high[i] & bit_mux) > 0;

                let pixel = ((p_high as u8) << 1) | (p_low as u8);

                if pixel != 0 {
                    let attr = self.sprite_attributes[i];
                    let palettes = (attr & 0x03) + 4;
                    let priority = (attr & 0x20) >> 5;
                    let is_sprite_0 = (i == 0) && self.sprite_0_hit_possible;

                    return(pixel, palettes, priority, is_sprite_0);
                }
            }
        }
        (0, 0, 0, false)
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
                self.oam[self.oam_addr as usize]
            }
            5 => {
                //cpu cant read from this register
                //TODO remember to make a read_function just for the PPU to read its own BUS
                panic!("Error, CPU tried to read PPU REGISTER 2005, which is for write only")
            }
            6 => {
                panic!("Error, CPU tried to read PPU REGISTER 2006, which is for write only")
            }
            7 => { // PPUDATA ($2007) - LEITURA

                // https://www.nesdev.org/wiki/PPU_registers#PPUDATA

                let mut data = self.ppubus.read_ppubus(self.internal_v.addr);
                
                if self.internal_v.addr >= 0x3F00 {
                    data = self.ppubus.read_ppubus(self.internal_v.addr);
                    self.ppu_data_buffer = self.ppubus.read_ppubus(self.internal_v.addr & 0x2FFF);
                } else {
                    let result = self.ppu_data_buffer;
                    self.ppu_data_buffer = data;
                    data = result;
                }

                if self.ppu_ctrl.contains(PpuCtrlFlags::IncrementVRAM) {
                    self.internal_v.addr = self.internal_v.addr.wrapping_add(32);
                } else {
                    self.internal_v.addr = self.internal_v.addr.wrapping_add(1);
                }
                data
            }
            _ => panic!("Error, an invalid address was called: {}", addr)
        }
    }

    pub fn write_registers(&mut self, addr: u16, data: u8)  -> bool {
        match addr {
            0 => { // PPUCTRL
                let before_nmi_status = self.ppu_ctrl.generate_vblank_nmi();
                self.ppu_ctrl = PpuCtrlFlags::from_bits_truncate(data);

                //updates the nametable's bits (bits 0-1 of data goes into bits 10-11 of t's)
                self.internal_t.set_nametable(data & 0x03);

                //Check for immediate NMI trigger
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
                self.oam[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
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

    //increments v.y, resets both x and y (horizontal and vertical line reset)
    fn update_scroll(&mut self) {
        if self.cycle == 256 && (self.scanline < 240 || self.scanline == 261) {
            self.internal_v.increment_fine_y();
        }

        if self.cycle == 257 && (self.scanline < 240 || self.scanline == 261) {
            self.internal_v.transfer_address_x(self.internal_t);
        }

        //resets y: copies t.y to v.y (vertical reset, happens during pre-render)
        if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 && ((self.ppu_mask & 0b0001_1000) != 0) {
            self.internal_v.transfer_address_y(self.internal_t);
        }
    }

    ///loads data into the shifters
    fn fetch_background(&mut self) {
        if self.scanline < 240 || self.scanline == 261 { 
            let visible_cycles = self.cycle > 0 && self.cycle <= 256;
            let pre_fetch_cycles = self.cycle >= 321 && self.cycle <= 336;

            if visible_cycles || pre_fetch_cycles {
                
                // Shift nos registradores
                self.bg_shifter_pattern_high    <<= 1;
                self.bg_shifter_pattern_low     <<= 1;
                self.bg_shifter_attribute_high  <<= 1;
                self.bg_shifter_attribute_low   <<= 1;

                // A cada 8 ciclos, carrega o próximo tile e incrementa Coarse X
                if self.cycle % 8 == 0 { 
                    self.swap_tile(); 
                }
            }
        }
    }

    fn load_sprites_to_next_line(&mut self) {
        if self.cycle == 257 {
            if self.scanline < 240 {
                self.sprite_evaluation();
                self.load_sprite_shifters();
            } else {
                self.sprite_count = 0;
            }
        }
    }
    
    ///RETURNS IF A NMI (Non-Maskable Interrupt) SHOULD BE ACTIVATED (which starts a VBLANK)
    ///the ppu displays a 256x240 resolution, even though the ppu works at a bigger resolution than that' 
    ///the nes ppu makes 340 cycles per scanline
    ///and 260 scanlines(one for each horizontal line of pixels)
    pub fn tick(&mut self, cycles: u16) -> bool {
        let mut nmi_triggered = false;

        for _ in 0..cycles {

            if self.scanline < 240 && self.cycle > 0 && self.cycle <= 256 {
                let bit_mux: u16 = 0x8000 >> self.fine_x;

                let p0_pixel = (self.bg_shifter_pattern_low & bit_mux) > 0;
                let p1_pixel = (self.bg_shifter_pattern_high & bit_mux) > 0;
                let bg_pixel = ((p1_pixel as u8) << 1) | (p0_pixel as u8);

                let p0_palette = (self.bg_shifter_attribute_low & bit_mux) > 0;
                let p1_palette = (self.bg_shifter_attribute_high & bit_mux) > 0;
                let palette_index = ((p1_palette as u8) << 1) | (p0_palette as u8);

                let (fg_pixel, fg_palette, fg_priority, is_sprite_zero) =
                    self.get_sprite_pixel_x(self.cycle - 1);

                let (pixel_final, palette_final) = match (bg_pixel, fg_pixel) {
                    (0, 0) => (0u8, 0u8),
                    (0, _) => (fg_pixel, fg_palette),
                    (_, 0) => (bg_pixel, palette_index),
                    _ => {
                        if is_sprite_zero {
                            let show_bg = (self.ppu_mask & 0b0000_1000) != 0;
                            let show_fg = (self.ppu_mask & 0b0001_0000) != 0;
                            if show_bg && show_fg && (self.cycle - 1) != 255 {
                                self.ppu_status.insert(PpuStatusFlags::Sprite0hit);
                            }
                        }
                        if fg_priority == 0 { (fg_pixel, fg_palette) }
                        else                 { (bg_pixel, palette_index) }
                    }
                };

                // Lê a paleta UMA vez, com os valores finais
                let color_index = self.ppubus.read_ppubus(
                    0x3F00 + (palette_final as u16 * 4) + pixel_final as u16
                );
                let color = NTSC_PALETTE[color_index as usize];

                let buf_offset = (self.scanline as usize * 256 + (self.cycle - 1) as usize) * 4;
                if buf_offset + 3 < self.frame_buffer.len() {
                    self.frame_buffer[buf_offset]     = color.r;
                    self.frame_buffer[buf_offset + 1] = color.g;
                    self.frame_buffer[buf_offset + 2] = color.b;
                    self.frame_buffer[buf_offset + 3] = 255;
                }
            }

            let render_enabled = (self.ppu_mask & 0b0001_1000) != 0;

            if render_enabled {
                self.fetch_background();

                self.update_scroll();

                self.load_sprites_to_next_line();
            }

            // updating cycles and scanlines
            self.cycle += 1;

            if self.scanline == 241 && self.cycle == 1 {
                self.ppu_status.insert(PpuStatusFlags::VblankFlag);
                self.frame_complete = true;
                self.frame_counter += 1;
                if self.ppu_ctrl.contains(PpuCtrlFlags::VblankNMI) {
                    nmi_triggered = true;
                }
            }

            if self.cycle > 340 {
                self.cycle = 0;
                self.scanline += 1;
                
                if self.scanline > 261 {
                    self.scanline = 0;
                    self.ppu_status.remove(PpuStatusFlags::VblankFlag);
                    self.ppu_status.remove(PpuStatusFlags::Sprite0hit);
                    self.ppu_status.remove(PpuStatusFlags::SpriteOverflow);
                    self.frame_complete = false;
                }
            }

            if self.scanline == 120 && self.cycle == 128 {
                println!("FRAME: {} | ADDR V: {:#06X} | CTRL: {:08b} | DATA: {:#04X}", 
                    self.frame_counter, // você precisaria de um contador de frames simples
                    self.internal_v.addr, 
                    self.ppu_ctrl.bits(),
                    self.bg_nametable_byte
                );
            }
        }
        nmi_triggered
    }

    fn get_sprites(&mut self) {
        if (257..=320).contains(&self.cycle) {
            self.sprites_in_scanline = 0;
            self.sprite_0_in_scanline = false;
            self.secondary_oam.fill(0xff);
        }
    }

    fn sprite_evaluation(&mut self) {
        self.secondary_oam.fill(0xFF); // <- ADICIONAR ISSO
        self.sprite_0_hit_possible = false; // <- E ISSO

        let mut sprite_count = 0;
        let sprite_height = if self.ppu_ctrl.contains(PpuCtrlFlags::SpriteSize) { 16 } else { 8 };

        for n in 0..64 {
            let y_coord = self.oam[n * 4];
            
            if self.scanline >= y_coord as u16 && self.scanline < (y_coord as u16 + sprite_height) {
                if sprite_count < 8 {
                    self.secondary_oam[sprite_count as usize * 4] = self.oam[n * 4];
                    self.secondary_oam[sprite_count as usize * 4 + 1] = self.oam[n * 4 + 1];
                    self.secondary_oam[sprite_count as usize * 4 + 2] = self.oam[n * 4 + 2];
                    self.secondary_oam[sprite_count as usize * 4 + 3] = self.oam[n * 4 + 3];

                    if n == 0 {
                        self.sprite_0_hit_possible = true;
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

        //searching the nametable ID (or TIle ID)
        self.bg_nametable_byte = self.ppubus.read_ppubus(self.internal_v.get_nametable_addr());

        //searching the attribute byte
        self.bg_attribute_byte = self.ppubus.read_ppubus(self.internal_v.get_attribute_addr());

        //takes the pattern low and high byte
        let pattern_addr = self.internal_v.get_pattern_table_addr(
            //self.ppu_ctrl.clone(), //TODO
            self.ppu_ctrl,
            self.bg_nametable_byte
        );
        //println!("{:#X}", pattern_addr);
        self.bg_low_byte = self.ppubus.read_ppubus(pattern_addr);
        self.bg_high_byte = self.ppubus.read_ppubus(pattern_addr + 8);

        //if self.bg_low_byte != 0 || self.bg_high_byte != 0{
        //    println!("Li um tile: {:X} / {:X}", self.bg_low_byte, self.bg_high_byte)
        //}

        //puts the data back into the shifters
        self.bg_shifter_pattern_low  |= self.bg_low_byte  as u16;
        self.bg_shifter_pattern_high |= self.bg_high_byte as u16;

        // repeat the attribute byte 8 times for each pixel bit
        let attribute_shift = (self.internal_v.get_coarse_y() & 0x02) * 2 + (self.internal_v.get_coarse_x() & 0x02);
        let attribute_palette = (self.bg_attribute_byte >> attribute_shift) & 0x03;

        self.bg_shifter_attribute_low = (self.bg_shifter_attribute_low & 0xFF00) | if (attribute_palette & 0b01) != 0 { 0xFF } else { 0x00 };
        self.bg_shifter_attribute_high = (self.bg_shifter_attribute_high & 0xFF00) | if (attribute_palette & 0b10) != 0 { 0xFF } else { 0x00 };
        
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