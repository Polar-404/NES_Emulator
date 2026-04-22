use sysinfo::{System, Pid};
use macroquad::prelude::get_time;
pub struct PerfomanceStats {
    sys: System,
    pid: Pid,

    pub cpu_usage: f32,
    pub main_emu_thread: f32,
    pub memory_usage_mb: f64,
    pub last_update: f64,
}
impl PerfomanceStats {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
            pid: sysinfo::get_current_pid().unwrap(),
            cpu_usage: 0.0,
            main_emu_thread: 0.0,
            memory_usage_mb: 0.0,
            last_update: get_time(),
        }
    }
    pub fn update_status(&mut self) {
        let current_time = get_time();
        if current_time - self.last_update >= 0.5 {
            self.sys.refresh_process(self.pid);
            if let Some(process) = self.sys.process(self.pid) {
                self.main_emu_thread = process.cpu_usage(); 
                self.cpu_usage = self.main_emu_thread / self.sys.cpus().len() as f32;
                self.memory_usage_mb = process.memory() as f64 / 1024.0 / 1024.0;
            } 
            self.last_update = current_time;
        }
    }
}