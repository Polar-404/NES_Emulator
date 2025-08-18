use core::panic;
use std::path::Path;
use crate::memory::mappers::*;
use crate::ppu::ppu::PPU;

use std::rc::Rc; // Importe Rc
use std::cell::RefCell;

pub struct BUS {

    //[https://www.nesdev.org/wiki/CPU_memory_map]

    cpu_memory: [u8; 0x0800],
    nes_apu_and_io_registers: [u8; 0x18],

    ///APU and I/O functionality that is normally disabled.
    apu_and_io_functionality: [u8; 0x08], 

    ///Unmapped. Available for cartridge use.
    ///[$6000–$7FFF | Usually cartridge RAM, when present]
    ///[$8000–$FFFF | Usually cartridge ROM and mapper registers]
    mapper: Rc<RefCell<Box<dyn Mapper>>>,
    ppu: PPU,
}
impl BUS {
    
    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        BUS {
            cpu_memory: [0; 0x0800],
            nes_apu_and_io_registers: [0; 0x18],
            apu_and_io_functionality: [0; 0x08], 
            mapper: Rc::clone(&mapper),
            ppu: PPU::new(mapper),
        }
    }
    
    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let addr = addr & 0x07FF;
                self.cpu_memory[addr as usize]
            }
            0x2000..=0x3FFF => {
                let addr: u8 = (addr & 0x07) as u8;
                self.ppu.read_registers(addr)
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
                self.mapper.borrow().read(addr)
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
                self.ppu.write_registers(addr, val);
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
                //passing it's real address(without subtraction) to the mapper to take care of it
                self.mapper.borrow_mut().write(addr, val)
            }
        }
    }
    #[allow(dead_code)]
    pub fn mem_read_u16(&mut self, pos: u16) -> u16{
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        return (hi << 8) | (lo as u16);
    }
    #[allow(dead_code)]
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

#[allow(dead_code)]
pub fn load_rom_from_file(path: &Path) -> Box<dyn Mapper>{

    //reads the entire content of a file into a vector of bytes(which is excatly what i need)
    let rom_data = std::fs::read(path).expect("Failed to extract ROM");
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
        2 => {
            todo!()
        }
        _ => panic!("The given mapper is not suported yet")
    }
}