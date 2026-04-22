use crate::memory::mapper_base::*;

use crate::memory::game_save::GameSave;

pub struct InesMapper163 {
    game_save: GameSave,

    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    chr_ram: Box<[u8]>,

    security_latch: bool,

    mirroring: Mirroring,

    // bank_registers: [u8; 8],
    bank_select: usize, 
}
impl InesMapper163 {
    pub fn new(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>, mirroring: Mirroring, game_save: GameSave) -> Self {
        let chr_ram = if chr_rom.is_empty() { vec![0; 8192].into() } else { vec![].into() };
        Self {
            game_save,
            prg_rom,
            chr_rom,
            chr_ram,

            mirroring,
            security_latch: false,
            bank_select: 0,
        }
    }
}
impl Mapper for InesMapper163 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x5000..=0x5FFF => {
                // Lógica de registradores (NÃO chame o game_save aqui)
                if addr == 0x5100 { 
                    if self.security_latch { 0 } else { 4 } 
                } else { 0 }
            }
            0x6000..=0x7FFF => {
                self.game_save.read(addr) 
            }
            0x8000..=0xFFFF => {
                let offset = (self.bank_select * 0x8000) + (addr as usize - 0x8000);
                self.prg_rom[offset % self.prg_rom.len()]
            }
            _ => 0
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x5000..=0x5FFF => {
                if addr == 0x5000{
                    self.mirroring = if (val & 0x01) == 0 {
                        Mirroring::Horizontal
                    } else {
                        Mirroring::Vertical
                    }
                }
                if addr == 0x5101 {
                    self.security_latch = (val & 0x01) != 0;
                }
                if (0x5000..=0x53ff).contains(&addr) {
                    self.bank_select = (val & 0x3F) as usize; 
                }
            }
            0x6000..=0x7FFF => self.game_save.write(addr - 0x6000, val),
            _ => {}
            _ => {}
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        if !self.chr_rom.is_empty() {
            self.chr_rom[addr as usize]
        } else {
            self.chr_ram[addr as usize]
        }
    }

    fn write_chr(&mut self, addr: u16, val: u8) {
        if self.chr_rom.is_empty() {
            self.chr_ram[addr as usize] = val;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
