use std::path::PathBuf;
use rfd::FileDialog;

pub fn open_rom_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("NES ROMs", &["nes"])
        .pick_file()
}