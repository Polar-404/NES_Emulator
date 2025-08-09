use crate::bus::Mapper;

pub struct TestMapper {
    prg_rom: [u8; 0x8000],
    chr_rom: [u8; 0x2000],
}

impl TestMapper {
    // Método de fábrica para criar uma nova instância
    pub fn new() -> Self {
        Self {
            prg_rom: [0; 0x8000],
            chr_rom: [0; 0x2000],
        }
    }
}

impl Mapper for TestMapper {
    fn read(&self, addr: u16) -> u8 {
        self.prg_rom[(addr - 0x8000) as usize]
    }
    fn write(&mut self, addr: u16, val: u8) {
        self.prg_rom[(addr - 0x8000) as usize] = val;
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }

    fn write_chr(&mut self, addr: u16, val: u8) {
        self.chr_rom[addr as usize] = val;
    }
}