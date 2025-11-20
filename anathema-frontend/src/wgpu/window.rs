use std::sync::Arc;

use anathema_geometry::Size;
use winit::application::ApplicationHandler;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowAttributes};

use crate::wgpu::font::{DEFAULT_FONT, Font};

use super::{GraphicsCtx, Renderer};

pub(crate) struct WindowHandler<Init, Tick> {
    window_attributes: WindowAttributes,
    ctx: Option<GraphicsCtx>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    init: Init,
    tick: Tick,
}

impl<Init, Tick> WindowHandler<Init, Tick> {
    pub fn new(init: Init, tick: Tick) -> Self {
        Self {
            window_attributes: WindowAttributes::default(),
            ctx: None,
            window: None,
            renderer: None,
            init,
            tick,
        }
    }
}

impl<Init, Tick> ApplicationHandler for WindowHandler<Init, Tick>
where
    Init: FnMut(&mut GraphicsCtx),
    Tick: FnMut(&mut GraphicsCtx),
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Ok(window) = event_loop.create_window(self.window_attributes.clone()) else {
            panic!("failed to create window")
        };

        let size = window.inner_size();

        let mut ctx = match pollster::block_on(GraphicsCtx::new(window)) {
            Ok(s) => s,
            Err(err) => panic!("{err}"),
        };
        (self.init)(&mut ctx);

        let font_texture = ctx.load_texture_from_bytes(DEFAULT_FONT);
        let size = Size::new(size.width as f32, size.height as f32);
        let renderer = Renderer::new(size, Font::new(font_texture));

        self.ctx.replace(ctx);
        self.renderer.replace(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(new_size) => {
                let Some(ctx) = &mut self.ctx else { return };
                let Some(renderer) = &mut self.renderer else { return };
                let (w, h) = (new_size.width, new_size.height);
                let size = Size::new(w as f32, h as f32);
                renderer.resize(size);
                ctx.resize(size);
            }
            winit::event::WindowEvent::RedrawRequested => {
                let Some(ctx) = &mut self.ctx else { return };
                let Some(renderer) = &mut self.renderer else { return };
                (self.tick)(ctx);
                ctx.window.request_redraw();
                renderer.render(ctx);
                // self.gameloop.tick(&mut self.renderer, graphics);
            }
            _ => {
                let Some(ctx) = &mut self.ctx else { return };
                // state.event(event, &mut self.server);
            }
        }
    }
}
