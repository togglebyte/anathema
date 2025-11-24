use std::sync::Arc;
use std::time::{Duration, Instant};

use anathema_geometry::Size;
use winit::application::ApplicationHandler;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowAttributes};

use super::temporary::Ctx;
use super::{GraphicsCtx, Renderer};
use crate::wgpu::font::{Font, DEFAULT_FONT};

/// Screen configuration
pub enum ScreenConfig {
    /// A cell has a fixed size, so the screen is divided by the cell size,
    /// and padding is applied
    CellSize(Size),
    /// A cell has the width of window_size.width / cols, and a height of window_size.height / rows
    CellCount { rows: usize, cols: usize },
}

pub(crate) struct WindowHandler<Tick> {
    window_attributes: WindowAttributes,
    ctx: Option<GraphicsCtx>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    tick: Tick,
    now: Instant,
    config: ScreenConfig,
}

impl<Tick> WindowHandler<Tick> {
    pub fn new(tick: Tick, config: ScreenConfig) -> Self {
        Self {
            window_attributes: WindowAttributes::default(),
            ctx: None,
            window: None,
            renderer: None,
            tick,
            now: Instant::now(),
            config,
        }
    }
}

impl<Tick> ApplicationHandler for WindowHandler<Tick>
where
    Tick: FnMut(f32, Ctx<'_>),
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Ok(window) = event_loop.create_window(self.window_attributes.clone()) else {
            panic!("failed to create window")
        };

        let size = window.inner_size();
        let size = Size::new(size.width as f32, size.height as f32);

        let mut ctx = match pollster::block_on(GraphicsCtx::new(window)) {
            Ok(s) => s,
            Err(err) => panic!("{err}"),
        };

        let font_texture = ctx.load_texture_from_bytes(DEFAULT_FONT, "default font");

        let char_size = match self.config {
            ScreenConfig::CellSize(size) => size,
            ScreenConfig::CellCount { rows, cols } => todo!(),
        };

        let font = {
            let font_size = Size::new(256.0, 32.0);
            let char_size = Size::new(6.0, 6.0);
            Font::new(font_texture, font_size, char_size)
        };

        let renderer = Renderer::new(size, font);

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
                let Some(graph_ctx) = &mut self.ctx else { return };
                let Some(renderer) = &mut self.renderer else { return };

                let ctx = Ctx::new(renderer, graph_ctx, &self.config);

                let dt = self.now.elapsed().as_micros() as f32;
                self.now = Instant::now();
                (self.tick)(dt, ctx);
                graph_ctx.window.request_redraw();
                renderer.render(graph_ctx);
                // self.gameloop.tick(&mut self.renderer, graphics);
            }
            _ => {
                let Some(ctx) = &mut self.ctx else { return };
                // state.event(event, &mut self.server);
            }
        }
    }
}
