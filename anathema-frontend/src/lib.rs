use anathema_geometry::{Pos, Region};

pub use self::brush::Brush;

pub mod crossterm;
mod screenbuffer;
pub mod wgpu;
mod brush;

pub trait Frontend {
    // This will happen outside of the widget and will be called for any widget 
    // that returns `true` for `wants_draw`
    fn apply_brush_to_region(&mut self, region: Region, brush: &dyn Brush);

    fn set_text(&mut self, pos: Pos, text: &str);
}
