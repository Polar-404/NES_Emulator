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

use crate::{memory::mappers::Mapper, ppu::ppubus::PPUBUS};
use std::rc::Rc;
use std::cell::RefCell;

bitflags! {
    #[derive(Debug)]
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

    pub oam_dma:    u8, //[0x4014] adress
    pub ppubus:     PPUBUS,
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
            oam_dma:    0,

            ppubus:     PPUBUS::new(mapper)
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
            _ => panic!("um endereço invalido foi chamado: {}", addr)
        }
    }

    pub fn write_registers(&mut self, addr: u16, data: u8) {
        match addr {
            0 => { // PPUCTRL
                let before_nmi_status = self.ppu_ctrl.generate_vblank_nmi();
                self.ppu_ctrl = PpuCtrlFlags::from_bits_truncate(data);

                // Check for immediate NMI trigger
                let after_nmi_status = self.ppu_ctrl.generate_vblank_nmi();
                
                // If NMI was just enabled AND the PPU is already in vblank, trigger an NMI
                if !before_nmi_status && after_nmi_status && self.ppu_status.contains(PpuStatusFlags::VblankFlag) {
                    // This `nmi_callback` needs to be defined to communicate with the CPU.
                    // You could add a field to the PPU struct or pass a callback.
                    // For simplicity, let's assume PPU can directly set the CPU's NMI flag.
                    // This requires the PPU to hold a reference to the CPU or a shared flag.
                    // A better approach is to make `tick` return a value.
                    // Let's stick with the `tick` return value approach you already have.
                    // So, this logic should be handled by the CPU after calling the write function.
                }
            }
            1 => {
                self.ppu_mask   = data
            }
            2 => {
                self.ppu_status = PpuStatusFlags::from_bits_truncate(data)
            }
            3 => {
                self.oam_addr   = data
            }
            4 => {
                self.oam_data   = data
            }
            5 => {
                self.ppu_scrl.write_byte(data);
            }
            6 => {
                self.ppu_addr.write_byte(data);
            }
            7 => {
                self.ppubus.write_ppubus(self.ppu_addr.value, data);
            }
            _ => panic!("um endereço invalido foi chamado: {}", addr)
        }
    }
    
    ///RETURNS IF A NMI (Non-Maskable Interrupt) SHOULD BE ACTIVATED (which starts a VBLANK)
    ///the ppu displays a 256x240 resolution, even though the ppu works at a bigger resolution than that' 
    ///the nes ppu makes 340 cycles per scanline
    ///and 260 scanlines(one for each horizontal line of pixels)
    pub fn tick(&mut self, cycles: u8) -> bool {
        for _ in 0..cycles {
            if self.scanline < 240 && self.cycle < 256 {
                let pixel_index = (self.scanline as usize * 256 + self.cycle as usize) * 4;
                if pixel_index + 3 < self.frame_buffer.len() {

                   //CHESS BOARD LOGIC
                   let color_val = if (self.cycle / 8 + self.scanline / 8) % 2 == 0 { 255 } else { 0 };
                   self.frame_buffer[pixel_index] = color_val;     // Red
                   self.frame_buffer[pixel_index + 1] = color_val; // Green
                   self.frame_buffer[pixel_index + 2] = color_val; // Blue
                   self.frame_buffer[pixel_index + 3] = 255;       // Alpha

                   
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
                }
            }
        }
        false
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