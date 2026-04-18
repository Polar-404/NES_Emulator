#[allow(dead_code)] //TODO temporario só pra ele parar de encher o saco

#[derive(Clone, Copy, Debug)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    SingleScreenLower,
    SingleScreenUpper
}

/// A memory mapper that abstracts over different NES cartridge board configurations.
///
/// NES cartridges use various mapper chips to extend the addressable memory beyond
/// the CPU's and PPU's native limits. Each mapper implements a different bank-switching
/// strategy for PRG ROM (program data) and CHR ROM (graphics data).
///
/// Implementing this trait allows the emulator to treat all cartridges uniformly,
/// regardless of their underlying mapper chip (e.g., NROM, MMC1, MMC3).
pub trait Mapper {
    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, val: u8);
    
    fn read_chr(&self, addr: u16) -> u8;
    
    fn write_chr(&mut self, addr: u16, val: u8);

    fn mirroring(&self) -> Mirroring;

    //optional (depends on the cartridge)
    fn irq_pending(&self) -> bool { false }
    fn acknowledge_irq(&mut self) {}
    fn notify_ppu_address(&mut self, _addr: u16) {}
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
    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr_ram: Box<[u8]>,

    mirroring: Mirroring,

    bank_select_register: u8,

    /// Array dos 8 registradores de banco do MMC3.
    /// O registrador a ser escrito é selecionado pelos bits RRR do Bank Select Register:
    ///
    /// ```text
    /// 7  bit  0
    /// ---- ----
    /// CPMx xRRR
    /// |||   |||
    /// |||   +++- Specify which bank register to update on next write to Bank Data register
    /// |||          000: R0: Select 2 KB CHR bank at PPU $0000-$07FF (or $1000-$17FF)
    /// |||          001: R1: Select 2 KB CHR bank at PPU $0800-$0FFF (or $1800-$1FFF)
    /// |||          010: R2: Select 1 KB CHR bank at PPU $1000-$13FF (or $0000-$03FF)
    /// |||          011: R3: Select 1 KB CHR bank at PPU $1400-$17FF (or $0400-$07FF)
    /// |||          100: R4: Select 1 KB CHR bank at PPU $1800-$1BFF (or $0800-$0BFF)
    /// |||          101: R5: Select 1 KB CHR bank at PPU $1C00-$1FFF (or $0C00-$0FFF)
    /// |||          110: R6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
    /// |||          111: R7: Select 8 KB PRG ROM bank at $A000-$BFFF
    /// ||+-------- Nothing on the MMC3, see MMC6
    /// |+--------- PRG ROM bank mode (0: $8000-$9FFF swappable, $C000-$DFFF fixed to second-last bank;
    /// |                              1: $C000-$DFFF swappable, $8000-$9FFF fixed to second-last bank)
    /// +---------- CHR A12 inversion (0: two 2 KB banks at $0000-$0FFF, four 1 KB banks at $1000-$1FFF;
    ///                                1: two 2 KB banks at $1000-$1FFF, four 1 KB banks at $0000-$0FFF)
    /// ```
    bank_registers: [u8; 8],
    bank_select: usize, 

    irq_counter: u8,
    irq_latch: u8,
    irq_enabled: bool,
    irq_pending: bool,    // <- flag que o bus vai ler
    irq_reload: bool,     // força recarga no próximo clock A12

    last_a12: bool,

    prg_ram_chip_enable: bool,     
    prg_ram_w_protection: bool,
}

impl InesMapper004 {
    pub fn new(prg_rom: Box<[u8]>, chr_rom: Box<[u8]>, mirroring: Mirroring) -> Self {
        let chr_ram = if chr_rom.is_empty() { vec![0; 8192].into() } else { vec![].into() };

        InesMapper004 {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; 8192].into_boxed_slice(),
            chr_ram,

            mirroring,

            bank_select_register: 0,
            bank_registers: [0; 8],
            bank_select: 0,

            //IRQ
            irq_counter: 0,
            irq_latch: 0,

            irq_enabled: false,
            irq_pending: false,
            irq_reload: false,  

            last_a12: false,

            prg_ram_chip_enable: false,
            prg_ram_w_protection: false,
        }
    }
    pub fn bank_switch(&self, addr: u16) -> usize {
        let prg_mode = (self.bank_select_register & 0x40) != 0;
        let total_banks = self.prg_rom.len() / 8192;

        match addr {
            0x8000..=0x9FFF => {
                if !prg_mode {
                    (self.bank_registers[6] as usize) * 8192 + (addr as usize & 0x1FFF)
                } else {
                    (total_banks - 2) * 8192 + (addr as usize & 0x1FFF)
                }
            }
            0xA000..=0xBFFF => {
                // R7 always maps here, in both modes
                (self.bank_registers[7] as usize) * 8192 + (addr as usize & 0x1FFF)
            }
            0xC000..=0xDFFF => {
                if !prg_mode {
                    // Modo 0: Penúltimo banco fixo aqui
                    (total_banks - 2) * 8192 + (addr as usize & 0x1FFF)
                } else {
                    // Modo 1: R6 mapeia aqui
                    (self.bank_registers[6] as usize) * 8192 + (addr as usize & 0x1FFF)
                }
            }
            0xE000..=0xFFFF => {
                (total_banks - 1) * 8192 + (addr as usize & 0x1FFF)
            }
            _ => 0
        }
    }
}

impl Mapper for InesMapper004 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize]
            }
            0x8000..=0xFFFF => {
                //calculates the actual offset within the ROM based on the MMC3 registers
                let absolute_offset = self.bank_switch(addr);

                // Security/Mirroring: The '%' operator ensures that the index never exceeds
                // the size of the loaded PRG_ROM, preventing 'out of bounds' panic
                // if the game requests a non-existent bank, it 'mirrors' it back to the beginning
                self.prg_rom[absolute_offset % self.prg_rom.len()]
            }
            _ => {0}
        }
    }

    /// *REGISTERS:* https://www.nesdev.org/wiki/MMC3#Registers
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            //https://www.nesdev.org/wiki/MMC3#Registers
            0x8000..=0x9FFF => {
                //even addr selects the bank register
                if addr % 2 == 0 {
                    self.bank_select_register = val;
                    self.bank_select = (val & 0x07) as usize;
                } 
                //odd addr will write at selected bank registers
                else {
                    self.bank_registers[self.bank_select] = val;
                }
            }
            0xA000..=0xBFFF => {
                //Nametable arrangement ($A000-$BFFE, even)
                if addr % 2 == 0 {
                    if val & 0x01 == 0 {
                        //nametable arrangement = horizontal
                    } else {
                        //nametable arrangement = vertical
                    }
                }
                //PRG RAM protect ($A001-$BFFF, odd)
                else {
                    //Write protection (0: allow writes; 1: deny writes)
                    self.prg_ram_w_protection = (val & 0b0100_0000) == 0;

                    //PRG RAM chip enable (0: disable; 1: enable)
                    self.prg_ram_chip_enable = (val & 0b1000_0000) == 0;
                }
            }
            0xC000..=0xDFFF => {
                //IRQ latch ($C000-$DFFE, even)
                if addr % 2 == 0 {
                    self.irq_latch = val
                }
                //IRQ reload ($C001-$DFFF, odd)
                else {
                    self.irq_counter = (self.irq_counter & 0x00) | self.irq_latch
                }
            }
            0xE000..=0xFFFF => {
                //IRQ disable ($E000-$FFFE, even)
                //Writing any value to this register will disable MMC3 interrupts AND acknowledge any pending interrupts.
                if addr & 2 == 0 {
                    self.irq_enabled = false;
                    self.acknowledge_irq();
                }
                //IRQ enable ($E001-$FFFF, odd)
                //Writing any value to this register will enable MMC3 interrupts.
                else {
                    self.irq_enabled = true;
                }
            }

            _ => {
                unreachable!()
            }
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write_chr(&mut self, addr: u16, val: u8) {
        todo!()
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn acknowledge_irq(&mut self) {
        todo!()
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    ///notifies the cartridge of the current scanline and updates if the cpu should trigger an "IRQ"
    fn notify_ppu_address(&mut self, addr: u16) {
        let a12 = addr & 0x1000 != 0;

        if a12 && !self.irq_enabled {
            if self.irq_counter == 0 || self.irq_reload {
                self.irq_counter = self.irq_latch;
                self.irq_reload = false;
            } else {
                self.irq_counter -= 1
            }

            if self.irq_counter == 0 && self.irq_enabled {
                self.irq_pending = true;
            }
        }

        self.last_a12= a12;
    }
}