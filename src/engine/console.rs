use std::sync::Mutex;

pub static TERMINAL: Mutex<Vec<LogMessage>> = Mutex::new(Vec::new());

pub enum LogType {
    Warning,
    Debug,
    Info,
}

pub struct LogMessage {
    pub log_type: LogType,
    pub log_msg: String,
}

pub fn print_logs(log_type: LogType, log_msg: impl Into<String>) {
    if let Ok(mut logs) = TERMINAL.lock() {
        logs.push(LogMessage { 
            log_type, 
            log_msg: log_msg.into()
        });
        if logs.len() > 1500 {
            logs.remove(0);
        }
    }
}