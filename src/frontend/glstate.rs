use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GlDisplay, GetGlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface}
};

use glutin_winit::DisplayBuilder;
use glow::HasContext;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::{
    event_loop::ActiveEventLoop, raw_window_handle::HasWindowHandle, window::Window
};

pub struct GLState {
    pub gl: Arc<glow::Context>,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
}
impl GLState {
    pub fn new(event_loop: &ActiveEventLoop, window: &Window) -> Self {
        let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(false);

        let display_builder = DisplayBuilder::new().with_window_attributes(None);

        let (_, gl_config) = display_builder
            .build(event_loop, template, |mut configs| configs.next().unwrap())
            .unwrap();

        // create a context
        let context_attrs = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(3, 3))))
        .build(Some(window.window_handle().unwrap().as_raw()));

        let gl_display = gl_config.display();
        let not_current_ctx = unsafe {
            gl_display.create_context(&gl_config, &context_attrs).unwrap()
        };

        let size = window.inner_size();
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.window_handle().unwrap().as_raw(),
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        );

        let gl_surface = unsafe {
            gl_display.create_window_surface(&gl_config, &surface_attrs).unwrap()
        };

        let gl = Arc::new(unsafe {
            glow::Context::from_loader_function_cstr(|s| {
                gl_display.get_proc_address(s) as *const _
            })
        });

        let gl_context = not_current_ctx.make_current(&gl_surface).unwrap();

        Self { gl, gl_context, gl_surface }
    }

    pub fn swap_buffers(&self) {
        self.gl_surface.swap_buffers(&self.gl_context).unwrap()
    }

    fn resize(&self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.gl_surface.resize(
                &self.gl_context, 
                NonZeroU32::new(width).unwrap(), 
                NonZeroU32::new(height).unwrap()
            );
        }
    }
}