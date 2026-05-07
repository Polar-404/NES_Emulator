use crate::memory::{
    game_save::GameSave, 
    mapper_base::{Mapper, Mirroring}
};


///https://www.nesdev.org/wiki/UxROM
pub struct InesMapper002 {
    game_save: GameSave,

    prg_rom: Box<[u8]>,

    ///**InesMapper002 / UxROM** doesn't have a chr_rom, instead, it uses a chr_ram(usually 8kb) 
    ///that starts empty and the game will write the tiles there before it tries to render the screen
    chr_ram: Box<[u8]>,

    mirroring: Mirroring,

    /// Bank select ($8000-$FFFF)
    /// ```text
    /// 7  bit  0
    /// ---- ----
    /// xxxx pPPP
    ///      ||||
    ///      ++++- Select 16 KB PRG ROM bank for CPU $8000-$BFFF
    ///           (UNROM uses bits 2-0; UOROM uses bits 3-0)
    /// ```
    /// Emulator implementations of iNES mapper 2 treat this as a full 8-bit bank select register, without bus conflicts. This allows the mapper to be used for similar boards that are compatible.
    /// 
    /// To make use of all 8-bits for a 4 MB PRG ROM, an NES 2.0 header must be used (iNES can only effectively go to 2 MB).
    /// 
    /// The original UxROM boards used by Nintendo were subject to bus conflicts, 
    /// and the relevant games all work around this in software. Some emulators (notably FCEUX) will have bus conflicts by default, 
    /// but others have none. NES 2.0 submappers were assigned to accurately specify whether the game should be emulated with bus conflicts.
    bank_select: u8,
}
impl InesMapper002 {
    pub fn new(prg_rom: Box<[u8]>, mirroring: Mirroring, game_save: GameSave) -> Self {
        
        Self {
            game_save,

            prg_rom,
            chr_ram: vec![0; 8192].into_boxed_slice(),

            mirroring,

            bank_select: 0,
        }
    }
}

impl Mapper for InesMapper002 {

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                self.game_save.read(addr)
            }
            0x8000..=0xBFFF => {
                let addr = (self.bank_select as usize * 0x4000) + (addr as usize - 0x8000);
                self.prg_rom[addr]
            } 
            0xC000..=0xFFFF => {
                let addr = (self.prg_rom.len() - 0x4000) + (addr as usize - 0xC000);
                self.prg_rom[addr]
            }
            _ => 0
        }

    }
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.game_save.write(addr, val);
            }
            0x8000..=0xFFFF => {
                self.bank_select = val;
            }
            _ => {}
        }
        
    }
    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_ram[addr as usize]
    }
    fn write_chr(&mut self, addr: u16, val: u8) {
        self.chr_ram[addr as usize] = val;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
