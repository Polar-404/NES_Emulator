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
    SingleScreenLower,
    SingleScreenUpper
}

pub trait Mapper {
    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, val: u8);
    
    fn read_chr(&self, addr: u16) -> u8;
    
    fn write_chr(&mut self, addr: u16, val: u8);

    fn mirroring(&self) -> Mirroring;
}

//---------------- MAPPERS LIST ----------------


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

// TO DO ADD DOCUMENTATION
pub struct InesMapper001 {
    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,

    chr_ram: Box<[u8]>,

    shift_register: u8,
    shift_counter: u8,
    
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}
impl InesMapper001 {
    pub fn new(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>) -> Self {
        let chr_ram = if chr_rom.is_empty() { vec![0; 8192].into() } else { vec![].into() };
        
        Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; 8192].into_boxed_slice(), // $6000-$7FFF
            chr_ram,
            
            shift_register: 0x10, // O MMC1 starts with 1 at his fourth bit
            shift_counter: 0,
            control: 0x0C, // PRG ROM in banks of 16kb
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0, 
        }
    }
    ///auxiliar fn to calculate the bank switchign of the prg rom
    fn prg_addr(&self, addr: u16) -> usize {
        let prg_mode = (self.control >> 2) & 0b11;
        let bank = self.prg_bank & 0x0F; // Pega só os 4 bits do banco
        let prg_size = self.prg_rom.len();

        match prg_mode {
            //32KB (takes the entire ROM and throws into mem, ignoring the lower bit of the bank)
            0 | 1 => {
                let bank32 = bank & 0xFE;
                ((bank32 as usize * 0x4000) + (addr - 0x8000) as usize) % prg_size
            }
            //16KB: (fix the FIRST bank at $8000 and changes bank at $C000)
            2 => {
                if addr >= 0x8000 && addr < 0xC000 {
                    (addr - 8000) as usize % prg_size
                } else {
                    ((bank as usize * 0x4000) + (addr - 0xC000) as usize) % prg_size
                }
            }
            //16KB: fix the LAST bank at $C000 and changes the bank at $8000
            3 => {
                if addr >= 0x8000 && addr < 0xC000 {
                    ((bank as usize * 0x4000) + (addr - 0x8000) as usize) % prg_size
                } else {
                    let last_bank_offset = prg_size - 0x4000;
                    (last_bank_offset + (addr - 0xC000) as usize) % prg_size
                }
            }
            _ => unreachable!()
        }
    }
    ///auxiliary fn to calculate the CHR addr, since it can be changed in entire blocks of 8kb
    /// or in 2 separated blocks of 4kb
    fn chr_addr(&self, addr: u16) -> usize {
        let chr_mode = (self.control >> 4) & 1;

        if chr_mode == 0 {
            let bank = self.chr_bank_0 & 0xFE;
            (bank as usize * 0x1000) + addr as usize
        } else {
            if addr <= 0x0FFF {
                (self.chr_bank_0 as usize * 0x1000) + addr as usize
            } else {
                (self.chr_bank_1 as usize * 0x1000) + (addr as usize - 0x1000)
            }
        }
    }
}
impl Mapper for InesMapper001 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize]
            }
            0x8000..=0xFFFF => {
                let mapped_addr = self.prg_addr(addr);
                self.prg_rom[mapped_addr % self.prg_rom.len() /* TODO */]
            }
            _ => 0
        }
    }
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;

                if addr == 0x6000 && val != 0x80 {
                    // Lê a string do endereço $6004 em diante até achar um byte nulo (0x00)
                    let mut text = String::new();
                    let mut i = 4; // Começa no índice 4 (que equivale a $6004)
                    
                    while i < self.prg_ram.len() && self.prg_ram[i] != 0 {
                        text.push(self.prg_ram[i] as char);
                        i += 1;
                    }
                    
                    println!("--- RESULTADO DO TESTE ---");
                    println!("Status Code: {:#04X}", val);
                    println!("Mensagem: \n{}", text);
                    println!("--------------------------");
                }
            
            }
            0x8000..=0xFFFF => {
                if val & 0x80 != 0 {
                    self.shift_register = 0x10;
                    self.shift_counter = 0;
                    self.control |= 0x0C
                } else {
                    self.shift_register = (self.shift_register >> 1) | ((val & 1) << 4);
                    self.shift_counter += 1;
                    if self.shift_counter == 5 {
                        let target = (addr >> 13) & 0b11;
                        match target {
                            0 => self.control = self.shift_register,
                            1 => self.chr_bank_0 = self.shift_register,
                            2 => self.chr_bank_1 = self.shift_register,
                            3 => self.prg_bank = self.shift_register,
                            _ => unreachable!()
                        }

                        // reseting for the next write
                        self.shift_register = 0x10;
                        self.shift_counter = 0;
                    }
                }
            }
            _ => {}
        }
    }
    fn read_chr(&self, addr: u16) -> u8 {
        let mapped_chr = self.chr_addr(addr);
        if !self.chr_rom.is_empty() {
            self.chr_rom[mapped_chr % self.chr_rom.len()]
        } else {
            self.chr_ram[mapped_chr % self.chr_ram.len()]
        }
    }
    fn write_chr(&mut self, addr: u16, val: u8) {
        if self.chr_rom.is_empty() {
            let mapped_addr = self.chr_addr(addr);
            let len = self.chr_ram.len();
            self.chr_ram[mapped_addr % len] = val;
        }
    }
    fn mirroring(&self) -> Mirroring {
        match self.control & 0b11 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!()
        }
    }
}


/// http://nesdev.org/wiki/MMC3
/// 
/// ### BANKS
/// 
/// CPU **$6000-$7FFF** 8 KB PRG RAM bank (optional)
/// 
/// CPU **$8000-$9FFF** (or $C000-$DFFF): 8 KB switchable PRG ROM bank
/// 
/// CPU **$A000-$BFFF** 8 KB switchable PRG ROM bank
/// 
/// CPU **$C000-$DFFF** (or $8000-$9FFF): 8 KB PRG ROM bank, fixed to the second-last bank
/// 
/// CPU **$E000-$FFFF** 8 KB PRG ROM bank, fixed to the last bank
/// 
/// PPU **$0000-$07FF** (or $1000-$17FF): 2 KB switchable CHR bank
/// 
/// PPU **$0800-$0FFF** (or $1800-$1FFF): 2 KB switchable CHR bank
/// 
/// PPU **$1000-$13FF** (or $0000-$03FF): 1 KB switchable CHR bank
/// 
/// PPU **$1400-$17FF** (or $0400-$07FF): 1 KB switchable CHR bank
/// 
/// PPU **$1800-$1BFF** (or $0800-$0BFF): 1 KB switchable CHR bank
/// 
/// PPU **$1C00-$1FFF** (or $0C00-$0FFF): 1 KB switchable CHR bank

pub struct InesMapper004 {

}

impl InesMapper004 {
    pub fn new(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>) -> Self {
        InesMapper004 {  }
    }
}

//impl Mapper for InesMapper004 {
//
//}