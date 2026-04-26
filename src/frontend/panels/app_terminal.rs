use crate::engine::console::{LogType, TERMINAL};
use egui::{Color32, RichText, ScrollArea, Ui};


pub struct ConsoleViewer {
    pub show_info: bool,
    pub show_warning: bool,
    pub show_debug: bool,
}
impl ConsoleViewer {
    pub fn new() -> Self {
        Self {
            show_info: true,
            show_warning: true,
            show_debug: false,
        }
    }

    pub fn render_terminal(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_info, "Info");
            ui.checkbox(&mut self.show_warning, "Warning");
            ui.checkbox(&mut self.show_debug, "Debug");

            if ui.button("Clear").clicked() {
                if let Ok(mut logs) = TERMINAL.lock() {
                    logs.clear();
                }
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if let Ok(logs) = TERMINAL.lock() {
                    for log in logs.iter() {
                        
                        let should_show = match log.log_type {
                            LogType::Info => self.show_info,
                            LogType::Warning => self.show_warning,
                            LogType::Debug => self.show_debug,
                        };

                        if should_show {
                            let color = match log.log_type {
                                LogType::Info => Color32::WHITE,
                                LogType::Warning => Color32::YELLOW,
                                LogType::Debug => Color32::LIGHT_GRAY,
                            };

                            ui.label(RichText::new(&log.log_msg).color(color));
                        }
                    }
                }
            }
        );
    }
}
