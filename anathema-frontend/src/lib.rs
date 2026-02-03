use anathema_geometry::{Pos, Region};
use unicode_width::UnicodeWidthStr;

pub use self::brush::Brush;

mod brush;
pub mod crossterm;

pub trait Frontend {
    // This will happen outside of the widget and will be called for any widget
    // that returns `true` for `wants_draw`
    fn apply_brush_to_region(&mut self, brush: &dyn Brush, region: Region);

    fn set_text(&mut self, text: &str, pos: Pos);

    fn invalidate_region(&mut self, region: Region);

    fn repeat_text(&mut self, text: &str, count: usize, mut pos: Pos) {
        let width = text.width();
        for i in 0..count / width {
            self.set_text(text, pos);
            pos.x += width as i32;
        }
    }
}
