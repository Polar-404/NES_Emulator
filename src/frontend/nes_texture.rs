use glow::HasContext;

pub struct NesTexture {
    pub gl_texture: glow::Texture,
    pub egui_texture_id: egui::TextureId,
}

impl NesTexture {
    pub fn new(gl: &glow::Context, egui_glow: &mut egui_glow::EguiGlow) -> Self {
        let gl_texture = unsafe {
            let tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0, glow::RGBA as i32, 256, 240, 0,
                glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&vec![0u8; 256 * 240 * 4])),
            );
            
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            
            gl.bind_texture(glow::TEXTURE_2D, None);
            tex
        };
        
        let egui_texture_id = egui_glow.painter.register_native_texture(gl_texture);

        Self { gl_texture, egui_texture_id }
    }

    pub fn update(&self, gl: &glow::Context, pixels: &[u8]) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.gl_texture));
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D, 0, 0, 0, 256, 240,
                glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }
}