use crate::memory::mapper_base::*;

use crate::memory::game_save::GameSave;

// TO DO ADD DOCUMENTATION
pub struct InesMapper001 {
    /// $6000-$7FFF
    game_save: GameSave, 

    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,

    chr_ram: Box<[u8]>,

    shift_register: u8,
    shift_counter: u8,
    
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}
impl InesMapper001 {
    pub fn new(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>, game_save: GameSave) -> Self {
        let chr_ram = if chr_rom.is_empty() { vec![0; 8192].into() } else { vec![].into() };
        
        Self {
            game_save,
            prg_rom,
            chr_rom,
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
                self.game_save.read(addr)
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
                self.game_save.write(addr, val);

                if addr == 0x6000 && val != 0x80 {

                    //testing

                    //let mut text = String::new();
                    //let mut i = 4;
                    //
                    //while i < self.prg_ram.len() && self.prg_ram[i] != 0 {
                    //    text.push(self.prg_ram[i] as char);
                    //    i += 1;
                    //}
                    //
                    //println!("--- TEST RESULT ---");
                    //println!("Status Code: {:#04X}", val);
                    //println!("Mensagem: \n{}", text);
                    //println!("--------------------------");
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