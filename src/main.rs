#![windows_subsystem = "windows"]

mod cpu;
mod memory;
mod ppu;
mod apu;
mod engine;
mod frontend;

#[cfg(feature = "debug_log")]
mod debug;

#[macro_use]
extern crate bitflags;

use winit::event_loop::EventLoop;
use frontend::app::App;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[Error] the program suffered a critical error:\n{}", info);
        let _ = std::fs::write(".log/crashlog.log", msg);
    }));

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}