use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets, Skin};
use crate::EmulatorInstance;
use crate::MULTIPLY_RESOLUTION;

pub fn create_customized_skin(res: f32) -> Skin {
    let bg_image = Image::gen_image_color(16, 16, Color::from_rgba(30, 32, 40, 245));
    let btn_bg_image = Image::gen_image_color(16, 16, Color::from_rgba(50, 56, 70, 255));
    let btn_hover_image = Image::gen_image_color(16, 16, Color::from_rgba(82, 139, 255, 255));

    let font_size = (18.0 * res) as u16; 
    let transparent_image = Image::gen_image_color(16, 16, Color::from_rgba(0, 0, 0, 0));

    let window_style = root_ui()
        .style_builder()
        .background(bg_image)
        .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
        .text_color(WHITE)
        .font_size(font_size)
        .build();

    let button_style = root_ui()
        .style_builder()
        .background(btn_bg_image.clone())
        .background_hovered(btn_hover_image)
        .background_margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
        .text_color(WHITE)
        .font_size(font_size)
        .build();

    let label_style = root_ui()
        .style_builder()
        .text_color(Color::from_rgba(200, 200, 200, 255))
        .font_size((14.0 * res) as u16)
        .build();

    let editbox_style = root_ui()
        .style_builder()
        .background(transparent_image) 
        .background_margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
        .text_color(WHITE)
        .font_size((0.0 * res) as u16)
        .build();

    Skin {
        window_style,
        button_style,
        label_style,
        editbox_style, // Adicionado aqui!
        ..root_ui().default_skin()
    }
}

pub fn render_pause_menu(emu: &mut EmulatorInstance, custom_skin: Option<&Skin>) -> bool {
    let mut return_to_menu: bool = false;
    let res = MULTIPLY_RESOLUTION as f32;

    if let Some(custom_skin) = custom_skin {
        root_ui().push_skin(custom_skin);
    } else {
        root_ui().push_skin(&root_ui().default_skin());
    }
    
    let win_w = 320.0 * res;
    let win_h = 280.0 * res;

    let win_x = (screen_width() - win_w) / 2.0;
    let win_y = (screen_height() - win_h) / 2.0;

    draw_texture_ex(&emu.ppu_texture, 0.0, 0.0, WHITE, DrawTextureParams {
        dest_size: Some(vec2(256.0 * 2.0 * res, 240.0 * 2.0 * res)),
        ..Default::default()
    });

    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 });

    emu.show_debug_info();

    widgets::Window::new(hash!(), vec2(win_x, win_y), vec2(win_w, win_h))
        .label(" PAUSE ")
        .ui(&mut *root_ui(), |ui| {

            let btn_w = 180.0 * res; 
            let btn_h = 35.0 * res;
            let x_centered = (win_w - btn_w) / 2.0; 
            
            if widgets::Button::new("CONTINUE")
                .position(vec2(x_centered, 40.0 * res))
                .size(vec2(btn_w, btn_h))
                .ui(ui) 
            {
                emu.is_paused = false;
            }
            
            ui.label(Some(vec2(x_centered - (30.0 * res), 115.0 * res)), "AUDIO SETTINGS:");
            
            let slider_w = win_w - (40.0 * res);
            widgets::Group::new(hash!(), vec2(slider_w, 40.0 * res))
                .position(vec2(30.0 * res, 125.0 * res))
                .ui(ui, |ui| {
                    ui.slider(hash!(), "VOL", 0.0..2.0, &mut emu.cpu.bus.apu.volume);
                });
            let volume = emu.cpu.bus.apu.volume;
            ui.label(Some(vec2(40.0 * res, 130.0 * res)), format!("{}%", (volume * 100.0).round()).as_str());

            if widgets::Button::new("QUIT TO MENU")
                .position(vec2(x_centered, 200.0 * res))
                .size(vec2(btn_w, btn_h))
                .ui(ui) 
            {
                return_to_menu = true;
            }
        });
    
    root_ui().pop_skin();
    return_to_menu
}