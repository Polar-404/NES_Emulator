use core::panic;
use std::path::Path;

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
pub trait Mapper {
    fn read(&self, addr: u16) -> u8;
    
    // Escreve um byte na ROM do programa (PRG ROM) ou em registradores do mapper.
    fn write(&mut self, addr: u16, val: u8);
    
    // Lê um byte da ROM de caracteres (CHR ROM) para a PPU.
    fn read_chr(&self, addr: u16) -> u8;
    
    // Escreve um byte na RAM de caracteres (CHR RAM) ou em registradores do mapper.
    fn write_chr(&mut self, addr: u16, val: u8);
}

pub struct InesMapper000 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>
}

impl Mapper for InesMapper000 {
    fn read(&self, addr: u16) -> u8 {
        self.prg_rom[(addr - 0x8000) as usize]
    }
    fn write(&mut self, _addr: u16, _val: u8) {

    }
    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }
    fn write_chr(&mut self, _addr: u16, _val: u8) {

    }
}


pub struct BUS {

    //[https://www.nesdev.org/wiki/CPU_memory_map]

    cpu_memory: [u8; 0x0800],
    ppu_registers: [u8; 0x08],
    nes_apu_and_io_registers: [u8; 0x18],

    ///APU and I/O functionality that is normally disabled.
    apu_and_io_functionality: [u8; 0x08], 

    ///Unmapped. Available for cartridge use.
    ///[$6000–$7FFF | Usually cartridge RAM, when present]
    ///[$8000–$FFFF | Usually cartridge ROM and mapper registers]
    mapper: Box<dyn Mapper>,
}
impl BUS {
    
    pub fn new(mapper: Box<dyn Mapper>) -> Self {
        BUS {
            cpu_memory: [0; 0x0800],
            ppu_registers: [0; 0x08],
            nes_apu_and_io_registers: [0; 0x18],
            apu_and_io_functionality: [0; 0x08], 
            mapper,
            // ppu: PPU:new()
        }
    }
    
    pub fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let addr = addr & 0x07FF;
                self.cpu_memory[addr as usize]
            }
            0x2000..=0x3FFF => {
                let addr = addr & 0x0007;
                self.ppu_registers[addr as usize]
            }
            0x4000..=0x4017 => {
                let addr = addr - 0x4000;
                self.nes_apu_and_io_registers[addr as usize]
            }
            0x4018..=0x401F => {
                let addr = addr - 0x4018;
                self.apu_and_io_functionality[addr as usize]
            }
            0x4020..=0xFFFF => {
                let addr = addr - 0x4020;
                self.mapper.read(addr)
                //self.unmapped[addr as usize]
            }
        }
    }

    pub fn mem_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let addr = addr & 0x07FF;
                self.cpu_memory[addr as usize] = val
            }
            0x2000..=0x3FFF => {
                let addr = addr & 0x0007;
                self.ppu_registers[addr as usize] = val
            }
            0x4000..=0x4017 => {
                let addr = addr - 0x4000;
                self.nes_apu_and_io_registers[addr as usize] = val
            }
            0x4018..=0x401F => {
                let addr = addr - 0x4018;
                self.apu_and_io_functionality[addr as usize] = val
            }
            0x4020..=0xFFFF => {
                let addr = addr - 0x4020;
                self.mapper.write(addr, val)
            }
        }
    }

    pub fn mem_read_u16(&mut self, pos: u16) -> u16{
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        return (hi << 8) | (lo as u16);
    }
    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8; 
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
    
    //pub fn load(&mut self, program: Vec<u8>) {
    //    self.unmapped[0x0000 .. (0x0000 + program.len())].copy_from_slice(&program[..]); //copia de src: program para self: memory
    //    self.mem_write_u16(0xFFFC,0x8000);
    //}
}

fn load_rom_from_file(path: &Path) -> Box<dyn Mapper>{
    //reads the entire content of a file into a vector of bytes(which is excatly what i need)
    let rom_data = std::fs::read(path).expect("erro ao extrair a ROM");
    let mapper_match = (rom_data[7] & 0xF0) | (rom_data[6] >> 4);

    match mapper_match {
        0 => {
            let program_size = rom_data[4] as usize * 0x4000; // the size of the PRG ROM may be 16kb or 32kb, 
            //that info is at byte 4 as [1 if 16kb and 2 if 32kb]

            let chr_size = rom_data[5] as usize * 0x2000; // then I take the chr size, which is at byte 5

            let prg_rom_end = 16 + program_size; //the header size is 16 bytes
            let prg_rom_data = rom_data[16..prg_rom_end].to_vec(); // mapping the actual game
            
            //The CHR ROM starts after the PRG ROM
            let chr_rom_data = rom_data[prg_rom_end..(prg_rom_end + chr_size)].to_vec();

            Box::new(InesMapper000 {
                prg_rom: prg_rom_data,
                chr_rom: chr_rom_data,
            })
        }
        1 => {
            panic!("Mapper 1 is not suported yet")
        }
        _ => panic!("The given mapper is not suported yet")
    }
}