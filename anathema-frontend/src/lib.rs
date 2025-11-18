use anathema_geometry::{Pos, Region};

pub use self::brush::Brush;

pub mod crossterm;
pub mod wgpu;
mod brush;

pub trait Frontend {
    // This will happen outside of the widget and will be called for any widget 
    // that returns `true` for `wants_draw`
    fn apply_brush_to_region(&mut self, brush: &dyn Brush, region: Region);

    fn set_text(&mut self, text: &str, pos: Pos);
}
