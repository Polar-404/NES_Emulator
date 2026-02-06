// Bytes	Description
// 0-3	Constant $4E $45 $53 $1A (ASCII "NES" followed by MS-DOS end-of-file)
// 4	Size of PRG ROM in 16 KB units
// 5	Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
// 6	Flags 6 – Mapper, mirroring, battery, trainer
// 7	Flags 7 – Mapper, VS/Playchoice, NES 2.0
// 8	Flags 8 – PRG-RAM size (rarely used extension)
// 9	Flags 9 – TV system (rarely used extension)
// 10	Flags 10 – TV system, PRG-RAM presence (unofficial, rarely used extension)
// 11-15	Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15)

// 76543210
// ||||||||
// |||||||+- Nametable arrangement: 0: vertical arrangement ("horizontal mirrored") (CIRAM A10 = PPU A11)
// |||||||                          1: horizontal arrangement ("vertically mirrored") (CIRAM A10 = PPU A10)
// ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
// |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
// ||||+---- 1: Alternative nametable layout
// ++++----- Lower nybble of mapper number

#[allow(dead_code)] //TODO temporario só pra ele parar de encher o saco

#[derive(Clone, Copy, Debug)]
pub enum Mirroring {
    Vertical,
    Horizontal,
}

pub trait Mapper {
    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, val: u8);
    
    fn read_chr(&self, addr: u16) -> u8;
    
    fn write_chr(&mut self, addr: u16, val: u8);

    fn mirroring(&self) -> Mirroring;
}

//---------------- MAPPERS LIST ----------------

pub struct InesMapper000 {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring
}

impl Mapper for InesMapper000 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let prg_len = self.prg_rom.len();
                let index = (addr - 0x8000) as usize;

                if prg_len == 0x8000 {
                    self.prg_rom[index]
                } else {
                    self.prg_rom[index % 0x4000]
                }
            }
            _ => 0, 
        }
    }

    fn write(&mut self, _addr: u16, _val: u8) {

    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }
    
    fn write_chr(&mut self, _addr: u16, _val: u8) {
        panic!("Tried to write to CHR ROM on Mapper 000!");
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

}