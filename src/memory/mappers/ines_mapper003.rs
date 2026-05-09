use crate::memory::mapper_base::{Mapper, Mirroring};

///https://www.nesdev.org/wiki/CNROM
pub struct InesMapper003 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr_rom: Box<[u8]>,

    mirroring: Mirroring,

    chr_bank: u8,
    
} impl InesMapper003 {
    ///for the CNROM, anything different from zero in the "has_ram" means the mapper has 2kb of ram mirrored from $6000 to $7FFF
    pub fn new(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>, mirroring: Mirroring, has_ram: u8) -> Self {
        let prg_ram = if has_ram != 0 { vec![0; 8192].into() } else { vec![].into() };
        Self {
            prg_rom,
            prg_ram,
            chr_rom,

            mirroring,

            chr_bank: 0,
        }
    }
}
impl Mapper for InesMapper003 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                if !self.prg_ram.is_empty() {
                    let index = (addr - 0x6000) as usize & 0x07FF; 
                    self.prg_ram[index]
                } else {
                    0
                }
            }
            0x8000..=0xFFFF => {
                let index = (addr - 0x8000) as usize % self.prg_rom.len();
                self.prg_rom[index]
            }
            _ => 0
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                if !self.prg_ram.is_empty() {
                    let index = (addr - 0x6000) as usize & 0x07FF;
                    self.prg_ram[index] = val;
                }
            }
            0x8000..=0xFFFF => {
                self.chr_bank = val & 0x03;
            }
            _ => { }
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_rom[(self.chr_bank as usize * 8192) + (addr as usize)]
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) {
        
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}