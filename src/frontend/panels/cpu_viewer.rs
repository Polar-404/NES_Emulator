use egui::Color32;

use crate::engine::instance::EmulatorInstance;

pub fn render_cpu_viewer<'a>(ui: &mut egui_dock::egui::Ui, _emulator_instance: &'a EmulatorInstance) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Not implemented yet")
        .size(20.0)
        .color(Color32::YELLOW)
        .strong());
    });
}