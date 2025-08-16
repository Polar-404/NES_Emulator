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

bitflags! {
    #[derive(Debug)]
    pub struct PpuCtrlFlags: u8 {
        const NameTable1              = 0b00000001; //Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const NameTable2              = 0b00000010; //Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const IncrementVRAM           = 0b00000100; //VRAM address increment per CPU read/write of PPUDATA (0: add 1, going across; 1: add 32, going down)
        const SpritePattern           = 0b00001000; //Sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; ignored in 8x16 mode)
        const BackGroundPattern       = 0b00010000; //Background pattern table address (0: $0000; 1: $1000) Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
        const SpriteSize              = 0b00100000; //Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
        const PPUMasterSlaveSelect   = 0b01000000; //PPU master/slave select
        const VblankNMI               = 0b10000000; //Vblank NMI enable (0: off, 1: on)
    }
}

pub struct PPU {

    ppu_ctrl:   PpuCtrlFlags,
    ppu_mask:   u8,
    ppu_status: u8,
    oam_addr:   u8,
    oam_data:   u8,
    ppu_scrl:   u8,
    ppu_addr:   u8,
    ppu_data:   u8,

    oam_dma:    u8, //[0x4014] adress

    ////32 byte pallete [16 for backgroudn 16 for foreground]
    //palette_ram: [u8; 0x20], 

    ////stores object atributes such as position, orientatiom pallete, etc...
    //oam: [u8; 0xff], //256 bytes(up to 64 sprites)

    //vram: [u8; 0x0800], // 2KB VRAM
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            ppu_ctrl:   PpuCtrlFlags::from_bits_truncate(0b0000_0000),
            ppu_mask:   0,
            ppu_status: 0,
            oam_addr:   0,
            oam_data:   0,
            ppu_scrl:   0,
            ppu_addr:   0,
            ppu_data:   0,
            oam_dma:    0,

            //TODO implementar isso no BUS
            // palette_ram: [0, 0x20],
            // oam:    [0, 0xff],
            // vram:   [0, 0x0800],
        }
    }


    pub fn read_registers(&self, addr: u8) -> u8 {
        match addr {
            0 => {
                self.ppu_ctrl.bits()
            }
            1 => {
                self.ppu_mask
            }
            2 => {
                self.ppu_status
            }
            3 => {
                self.oam_addr
            }
            4 => {
                self.oam_data
            }
            5 => {
                self.ppu_scrl
            }
            6 => {
                self.ppu_addr
            }
            7 => {
                self.ppu_data
            }
            _ => panic!("um endereço invalido foi chamado: {}", addr)
        }
    }

    pub fn write_registers(&mut self, addr: u16, data: u8) {
        match addr {
            0 => {
                self.ppu_ctrl = PpuCtrlFlags::from_bits_truncate(data)
            }
            1 => {
                self.ppu_mask   = data
            }
            2 => {
                self.ppu_status = data
            }
            3 => {
                self.oam_addr   = data
            }
            4 => {
                self.oam_data   = data
            }
            5 => {
                self.ppu_scrl   = data
            }
            6 => {
                self.ppu_addr   = data
            }
            7 => {
                self.ppu_data   = data
            }
            _ => panic!("um endereço invalido foi chamado: {}", addr)
        }
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