use crate::memory::mappers::Mapper;
use std::rc::Rc; // Importe Rc
use std::cell::RefCell;

struct Vram {
    vram_state: u8
}

pub struct PPUBUS {
    //32 byte pallete [16 for backgroudn 16 for foreground]
    palette_ram: [u8; 0x20],
    //stores object atributes such as position, orientatiom pallete, etc...
    oam: [u8; 0xff], //256 bytes(up to 64 sprites
    vram: [u8; 0x0800], 
    mapper: Rc<RefCell<Box<dyn Mapper>>>// 2KB VRAM
}
impl PPUBUS {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> PPUBUS {
        PPUBUS {
            palette_ram: [0; 0x20],
            oam: [0; 0xff],
            vram: [0; 0x0800],
            mapper,
        }
    }
    pub fn write_vram(&mut self, addr: u16, data: u8) {
        let mut mapped_addr = addr;

        if mapped_addr >= 0x2000 && mapped_addr < 0x3000 {
            mapped_addr = mapped_addr - 0x2000;
            mapped_addr = mapped_addr % 0x0800;
        }
        
        if mapped_addr >= 0x3F00 && mapped_addr < 0x4000 {
            mapped_addr = mapped_addr % 0x20;
        }
        
        self.vram[mapped_addr as usize] = data;
    }
}
