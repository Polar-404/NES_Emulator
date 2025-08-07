pub struct BUS {

    //[https://www.nesdev.org/wiki/CPU_memory_map]

    cpu_memory: [u8; 0x7FF],
    ppu_registers: [u8; 0x08],
    nes_apu_and_io_registers: [u8; 0x18],

    ///APU and I/O functionality that is normally disabled.
    apu_and_io_functionality: [u8; 0x08], 

    ///Unmapped. Available for cartridge use.
    ///[$6000–$7FFF | Usually cartridge RAM, when present]
    ///[$8000–$FFFF | Usually cartridge ROM and mapper registers]
    unmapped: [u8; 0xBFE0] 

}
impl BUS {
    
    pub fn new() -> Self {
        BUS {
            cpu_memory: [0; 0x7FF],
            ppu_registers: [0; 0x08],
            nes_apu_and_io_registers: [0; 0x18],
            apu_and_io_functionality: [0; 0x08], 
            unmapped: [0; 0xBFE0]
            // ppu: PPU:new()
        }
    }
    pub fn mem_read(&self, addr: u16) -> u8 {
        if addr <= 2048 {
            let addr = addr & 0x7FF;
            return self.cpu_memory[addr as usize]
        }
        if addr <= 0x3FFF {
            let addr = addr & 0x08;
            return self.ppu_registers[addr as usize]
        }
        if addr <= 0x4017 {
            let addr = addr & 0x18;
            return self.nes_apu_and_io_registers[addr as usize]
        }
        if addr <= 0x401F {
            let addr = addr & 0x18;
            return self.apu_and_io_functionality[addr as usize]
        }
        self.unmapped[addr as usize]
    }
}