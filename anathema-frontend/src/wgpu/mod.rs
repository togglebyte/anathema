use anathema_compiler::Color;
use anathema_geometry::Pos;
use winit::event_loop::EventLoop;

use crate::wgpu::fonts::Char;

pub use self::ctx::GraphicsCtx;
pub use self::material::MaterialId;
pub use self::renderer::Renderer;
pub use self::sprite::{Sprite, SpriteId};
pub use self::window::ScreenConfig;

static DEFAULT_SHADER: &'static str = include_str!("shader.wgsl");

// TODO: remove this once we have some kind of runtime
pub fn justatest<Init, Tick>(config: ScreenConfig, init: Init, tick: Tick)
where
    Init: Fn(&mut GraphicsCtx),
    Tick: FnMut(f32, temporary::Ctx<'_>),
{
    let event_loop = EventLoop::new().unwrap();
    let mut app = window::WindowHandler::new(init, tick, config);
    event_loop.run_app(&mut app).unwrap();
}

mod buffer;
mod camera;
mod ctx;
mod error;
mod fonts;
mod material;
mod model;
mod renderer;
mod sprite;
mod texture;
mod window;

mod temporary;

#[derive(Debug, Default, Clone, PartialEq)]
enum State {
    #[default]
    Empty,
    Continuation,
    Char(char),
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Cell {
    style: Style,
    state: State,
}

impl Cell {
    pub fn space() -> Self {
        Self {
            style: Style::reset(),
            state: State::Empty,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Style {
    /// Foreground colour.
    pub fg: Option<Color>,
    /// Background colour.
    pub bg: Option<Color>,
    pub material: Option<MaterialId>,
}

impl Style {
    fn merge(&mut self, other: Style) {
        if let Some(fg) = other.fg {
            self.fg = Some(fg);
        }

        if let Some(bg) = other.bg {
            self.bg = Some(bg);
        }

        if let Some(mat) = other.material {
            self.material.replace(mat);
        }
    }

    fn reset() -> Self {
        Self::default()
    }
}
