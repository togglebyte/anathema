use winit::event_loop::EventLoop;

pub use self::ctx::GraphicsCtx;
pub use self::renderer::Renderer;

static DEFAULT_SHADER: &'static str = include_str!("shader.wgsl");

// TODO: remove this once we have some kind of runtime
pub fn justatest<Init, Tick>(init: Init, tick: Tick)
where
    Init: FnMut(&mut GraphicsCtx),
    Tick: Fn(&mut GraphicsCtx),
{
    let event_loop = EventLoop::new().unwrap();
    let mut app = window::WindowHandler::new(init, tick);
    event_loop.run_app(&mut app).unwrap();
}

mod ctx;
mod error;
mod material;
mod maths;
mod model;
mod renderer;
mod sprite;
mod texture;
mod window;
