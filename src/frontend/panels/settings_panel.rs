use crate::{engine::config::EmulatorConfig, ppu::palettes::PaletteTheme};

pub fn render_settings(settings: &mut EmulatorConfig, ui: &mut egui_dock::egui::Ui) {
    change_palette(settings, ui);

    ui.separator();

    ui.checkbox(&mut settings.hide_overscan, "Hide hide_overscan");
    ui.separator();

    ui.menu_button("Audio", |ui| {
         ui.add(egui::Slider::new(&mut settings.volume, 0.0_f32..=200.0_f32))
    });
}

fn change_palette(settings: &mut EmulatorConfig, ui: &mut egui_dock::egui::Ui) {
    let selected_text = match &settings.palette {
        PaletteTheme::Custom(name, _) => name.clone(),
        other => format!("{:?}", other),
    };
    egui::ComboBox::from_label("Palette")
    .selected_text(selected_text)
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut settings.palette, PaletteTheme::DefaultNtsc, "NTSC");
        ui.selectable_value(&mut settings.palette, PaletteTheme::Nestopia,    "Nestopia");
        ui.selectable_value(&mut settings.palette, PaletteTheme::Fceux,   "Composite");
        ui.selectable_value(&mut settings.palette, PaletteTheme::ShovelKnight, "ShovelKnight");
        ui.selectable_value(&mut settings.palette, PaletteTheme::SonyCxa, "SonyCxa");
        ui.selectable_value(&mut settings.palette, PaletteTheme::InvertedNtsc, "InvertedNtsc");

        if !settings.custom_palettes.is_empty() {
            for (name, colors) in &settings.custom_palettes {
                let is_selected = match &settings.palette {
                    PaletteTheme::Custom(current_name, _) => current_name == name,
                    _ => false,
                };

                if ui.selectable_label(is_selected, name).clicked() {
                    settings.palette = PaletteTheme::Custom(name.clone(), colors.clone());
                }
            }
        }

        ui.separator();
        
        if ui.button("Select Custom Palette").clicked() {
            if let Some(path) = rfd::FileDialog::new()
            .add_filter("Nes Palette", &["pal"])
            .pick_file() {
                if let Ok(colors_array) = PaletteTheme::create_palette_from_dot_pal(&path) {
                    let colors_vec = colors_array.to_vec();

                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    settings.custom_palettes.insert(name.clone(), colors_vec.clone());
                    settings.palette = PaletteTheme::Custom(name, colors_vec);
                    settings.save();
                } else {
                    eprintln!("Palette file invalid or corrupted (the file must be exactly 192 bytes)");
                }
            }
        }
    });
}