mod cpu;
mod memory;
mod ppu;
mod apu;
mod ui;
mod engine;

#[cfg(feature = "debug_log")]
mod debug;

#[macro_use]
extern crate bitflags;

use winit::event_loop::EventLoop;
use ui::app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}