use std::path::Path;

use crate::memory::{game_save::GameSave, mapper_base::{Mapper, Mirroring}};

pub enum CpuRam {
    Volatile([u8; 0x2000]),
    Persistent(GameSave),
}

///https://www.nesdev.org/wiki/CNROM
pub struct InesMapper003 {
    prg_rom: Box<[u8]>,
    prg_ram: CpuRam,
    chr_rom: Box<[u8]>,

    mirroring: Mirroring,

    chr_bank: u8,
    
} impl InesMapper003 {
    ///for the CNROM, anything different from zero in the "has_ram" means the mapper has 2kb of ram mirrored from $6000 to $7FFF
    pub fn new<P: AsRef<Path>>(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>, mirroring: Mirroring, has_save: Option<P>) -> Self {
        let prg_ram = if let Some(path) = has_save { 
            CpuRam::Persistent(GameSave::new(path))
        } else { 
            CpuRam::Volatile([0; 0x2000]) 
        };

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
                match &self.prg_ram {
                    CpuRam::Persistent(save) => {
                        save.read(addr)
                    }
                    CpuRam::Volatile(data) => {
                        if data.is_empty() {
                            let index = (addr - 0x6000) as usize & 0x07FF; 
                            data[index]
                        } else {
                            0
                        }
                    }
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
                match &mut self.prg_ram {
                    CpuRam::Persistent(save) => {
                        save.write(addr, val)
                    }
                    CpuRam::Volatile(data) => {
                        if !data.is_empty() {
                            let index = (addr - 0x6000) as usize & 0x07FF;
                            data[index] = val;
                        }
                    }
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