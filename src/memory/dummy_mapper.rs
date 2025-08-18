#![allow(unused_variables)]

use std::{cell::RefCell, rc::Rc};

use crate::memory::mappers::Mapper;

pub struct TestMapper {
    ram: [u8; 0x0800],
    prg_rom: Vec<u8>,
}

impl TestMapper {
    pub fn new(program: Vec<u8>) -> Rc<RefCell<Box<dyn Mapper>>>{
        let ram = [0; 0x0800];
        
        let mut prg_rom_vec = vec![0; 0x8000];
        let program_len = program.len();
        prg_rom_vec[0..program_len].copy_from_slice(&program[..]);
        
        let reset_vector = 0x8000;
        prg_rom_vec[0x7ffc] = (reset_vector & 0xff) as u8;
        prg_rom_vec[0x7ffd] = (reset_vector >> 8) as u8;

        Rc::new(
            RefCell::new(
                Box::new(Self {
                    ram,
                    prg_rom: prg_rom_vec,
        })))
    }
}

impl Mapper for TestMapper {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x8000..=0xFFFF => self.prg_rom[(addr - 0x8000) as usize],
            _ => 0,
        }
    }
    
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = val,

            _ => {},
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        0
    }
    fn write_chr(&mut self, addr: u16, val: u8) {}
}