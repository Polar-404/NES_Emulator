use crate::memory::mappers::{Mapper, Mirroring};

use std::rc::Rc; // Importe Rc
use std::cell::RefCell;

pub struct PPUBUS {
    //32 byte pallete [16 for backgroudn 16 for foreground]
    palette_ram: [u8; 0x20],
    //stores object atributes such as position, orientatiom pallete, etc...
    oam: [u8; 0xff], //256 bytes(up to 64 sprites
    vram: [u8; 0x0800], 
    pub mapper: Rc<RefCell<Box<dyn Mapper>>>, // 2KB VRAM

    pub last_read_palette: u8
}
impl PPUBUS {
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> PPUBUS {
        PPUBUS {
            palette_ram: [0; 0x20],
            oam: [0; 0xff],
            vram: [0; 0x0800],
            mapper,
            last_read_palette: 0
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
                let mut palette_addr = (addr & 0x1F) as usize;
                    
                // espelhamento: endereços de sprites 0x10, 0x14, 0x18, 0x1C 
                //apontam para os endereços de background 0x00, 0x04, 0x08, 0x0C
                if palette_addr >= 0x10 && palette_addr % 4 == 0 {
                    palette_addr -= 0x10;
                }
                //println!("data: {:#X}", data);
                self.palette_ram[palette_addr] = data;
            }
            _ => {
                todo!();
            }
        }
    }
    //TODO remover a referencia mutavel e limpar esse codigo depois que funcionar
    pub fn read_ppubus(&mut self, addr: u16) -> u8{
        match addr {
            //pattern tables(storages the tiles(8x8 pixels blocks))
            //THE MAJORITY OF THE CHR-ROM(PATTERN TABLES) IS READ-ONLY, 
            //BUT SOME OF THEM CAN BE WRITTEN, SO IT'S A GOOD PRATICE TO ALLOW IT TO BE POSSIBLE
            0..=0x1FFF => {
                self.mapper.borrow().read_chr(addr)
            }
            //VRAM (or nametable)
            0x2000..=0x3EFF  => {
                self.read_vram(addr)
            }
            0x3F00..=0x3FFF => {
                //mirroring the last addr of the palletes
                let mut palette_addr = (addr & 0x1F) as usize;

                //espelhamento de transparência: 0x10, 0x14, 0x18, 0x1C espelham para 0x00, 0x04, 0x08, 0x0C
                //e os índices 0x04, 0x08, 0x0C espelham para 0x00 na renderização.
                if palette_addr >= 0x10 && palette_addr % 4 == 0 {
                    palette_addr -= 0x10;
                }

                self.last_read_palette = self.palette_ram[palette_addr].clone();
                //println!("{:#X}",self.last_read_palette);
                
                self.palette_ram[palette_addr]
                
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
