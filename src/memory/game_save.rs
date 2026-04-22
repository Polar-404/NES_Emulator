use std::{env, fs, path::{Path, PathBuf}};

pub struct GameSave {
    file_path: PathBuf,
    save_data: [u8; 0x2000],
    sram_enabled: bool,
}
impl GameSave {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let save_file_path = Self::get_save_path(path.as_ref());

        let mut save = Self {
            file_path: save_file_path,
            save_data: [0; 0x2000],
            sram_enabled: true,
        };
        save.load_save_file();
        save
    }

    fn get_save_path(rom_path: &Path) -> PathBuf {
        let game_name = rom_path.file_stem().unwrap_or_default();

        let mut save_path = if cfg!(debug_assertions) {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".saves")
        } else {
            if let Ok(mut exec_path) = env::current_exe() {
                exec_path.pop();
                exec_path.push(".saves");
                exec_path
            } else {
                PathBuf::from("./.saves")
            }
        };

        let _ = fs::create_dir_all(&save_path);

        save_path.push(game_name);
        save_path.set_extension("sav");

        save_path
    }

    pub fn load_save_file(&mut self) {
        if let Ok(file) = fs::read(&self.file_path) {
            if file.len() == 0x2000 {
                self.save_data.copy_from_slice(&file);
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if self.sram_enabled {
            self.save_data[(addr - 0x6000) as usize]
        } else {
            0
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if self.sram_enabled {
            self.save_data[(addr - 0x6000) as usize] = data
        }
    }

    pub fn save_to_disk(&self) {
        let _ = fs::write(&self.file_path, &self.save_data);
    }
}

impl Drop for GameSave {
    fn drop(&mut self) {
        self.save_to_disk();
    }
}