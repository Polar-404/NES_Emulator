use crate::memory::bus::Mirroring;
use crate::memory::mappers::Mapper;
use std::rc::Rc; // Importe Rc
use std::cell::RefCell;

pub struct PPUBUS {
    //32 byte pallete [16 for backgroudn 16 for foreground]
    palette_ram: [u8; 0x20],
    //stores object atributes such as position, orientatiom pallete, etc...
    oam: [u8; 0xff], //256 bytes(up to 64 sprites
    vram: [u8; 0x0800], 
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>// 2KB VRAM
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

    pub fn write_ppubus(&mut self, addr: u16, data: u8) {
        match addr {
            //pattern tables(storages the tiles(8x8 pixels blocks))
            //THE MAJORITY OF THE CHR-ROM(PATTERN TABLES) IS READ-ONLY, 
            //BUT SOME OF THEM CAN BE WRITTEN, SO IT'S A GOOD PRATICE TO ALLOW IT TO BE POSSIBLE
            0..=0x1FFF => {
                self.mapper.borrow_mut().write(addr, data);
            }
            //VRAM (or nametable)
            0x2000..=0x3EFF  => {
                self.write_vram(addr, data);
            }
            0x3F00..=0x3FFF => {
                //mirroring the last addr of the palletes
                let addr = if(addr & 0x1F) >= 10 && (addr & 0x1F) % 4 == 0 {0x00} else {addr & 0x1F};
                self.palette_ram[addr as usize] = data;
            }


            _ => {
                todo!();
            }
        }
    }
    pub fn read_ppubus(&self, addr: u16) -> u8{
        match addr {
            //pattern tables(storages the tiles(8x8 pixels blocks))
            //THE MAJORITY OF THE CHR-ROM(PATTERN TABLES) IS READ-ONLY, 
            //BUT SOME OF THEM CAN BE WRITTEN, SO IT'S A GOOD PRATICE TO ALLOW IT TO BE POSSIBLE
            0..=0x1FFF => {
                self.mapper.borrow().read(addr)
            }
            //VRAM (or nametable)
            0x2000..=0x3EFF  => {
                self.read_vram(addr)
            }
            0x3F00..=0x3FFF => {
                //mirroring the last addr of the palletes
                let addr = if(addr & 0x1F) >= 10 && (addr & 0x1F) % 4 == 0 {0x00} else {addr & 0x1F};
                self.palette_ram[addr as usize]
            }


            _ => {
                todo!();
            }
        }
    }

    fn write_vram(&mut self, addr: u16, data: u8) {
        let mut addr = (addr - 0x2000) & 0x0FFF;

        match self.mapper.borrow().mirroring() {
            Mirroring::Vertical => {
                addr &= 0x07FF;
            }
            Mirroring::Horizontal => {
                addr = (addr & 0x03FF) + ((addr & 0x0800) >> 1);
                
            }
            //TODO didnt implement OneScreen mirroring yet
            #[allow(unreachable_patterns)]
            _ => {
                //with OneScreen mirroring all the nametables point to the same 1kb space 
                addr &= 0x03FF;
            }

        }
        
        self.vram[addr as usize] = data;
    }

    fn read_vram(&self, addr: u16) -> u8 {
        let mut addr = (addr - 0x2000) & 0x0FFF;

        match self.mapper.borrow().mirroring() {
            Mirroring::Vertical => {
                addr &= 0x07FF;
            }
            Mirroring::Horizontal => {
                addr = (addr & 0x03FF) + ((addr & 0x0800) >> 1);
                
            }
            //TODO didnt implement OneScreen mirroring yet
            #[allow(unreachable_patterns)]
            _ => {
                //with OneScreen mirroring all the nametables point to the same 1kb space 
                addr &= 0x03FF;
            }

        }
        
        self.vram[addr as usize]
    }
}
