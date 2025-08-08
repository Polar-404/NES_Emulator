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
    unmapped: [u8; 0xBFE0] 
}
impl BUS {
    
    pub fn new() -> Self {
        BUS {
            cpu_memory: [0; 0x0800],
            ppu_registers: [0; 0x08],
            nes_apu_and_io_registers: [0; 0x18],
            apu_and_io_functionality: [0; 0x08], 
            unmapped: [0; 0xBFE0]
            // ppu: PPU:new()
        }
    }
    //fn match_address(&mut self, addr: u16) -> &mut u8 {
    //}
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
                self.unmapped[addr as usize]
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
                self.unmapped[addr as usize] = val
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
    
    pub fn load(&mut self, program: Vec<u8>) {
        self.unmapped[0x0000 .. (0x0000 + program.len())].copy_from_slice(&program[..]); //copia de src: program para self: memory
        self.mem_write_u16(0xFFFC,0x8000);
    }
}