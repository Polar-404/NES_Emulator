use crate::memory::mapper_base::*;

// TO DO ADD DOCUMENTATION
pub struct InesMapper000 {
    pub prg_rom: Box<[u8]>,
    pub chr_rom: Box<[u8]>,
    pub prg_ram: Box<[u8]>,
    pub mirroring: Mirroring
}

impl InesMapper000 {
    pub fn new(prg_rom_data: Box<[u8]>, chr_rom_data: Box<[u8]>, mirroring: Mirroring) -> Self {
        InesMapper000 {
            prg_rom: prg_rom_data,
            chr_rom: chr_rom_data,
            prg_ram: vec![0; 8192].into_boxed_slice(),
            mirroring: mirroring
        }
    }
}
//the ability to read at 0x6000 and write at all, isnt something used on the mapper 0 
//but as far as I understood, blergg tests may use it so its there 

impl Mapper for InesMapper000 {

    fn read(&self, addr: u16) -> u8 {
        match addr {
            //0x6000..=0x7FFF => {
            //    self.prg_ram[(addr - 0x6000) as usize]
            //}
            // PRG-ROM
            0x8000..=0xFFFF => {
                let prg_len = self.prg_rom.len();
                let index = (addr - 0x8000) as usize;
                if prg_len == 0x8000 {
                    self.prg_rom[index]
                } else {
                    self.prg_rom[index % 0x4000]
                }
            }
            _ => {
                println!("Non mapped address, this might be a bug");
                0
            }
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;
            }
            0x8000..=0xFFFF => {
                
            }
            _ => {
                println!("Non mapped address write at {:#06x}", addr)
            }
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        if self.chr_rom.is_empty() {
            return 0;
        }
        self.chr_rom[addr as usize]
    }
    
    fn write_chr(&mut self, _addr: u16, _val: u8) {
        
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

}
