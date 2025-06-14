use anathema_geometry::{Pos, Size};
use anathema_value_resolver::Attributes;
use anathema_widgets::paint::Glyph;
use anathema_widgets::{GlyphMap, Style, WidgetRenderer};

#[derive(Debug)]
pub struct TestSurface {
    pub(super) size: Size,
    lines: Vec<Glyph>,
}

impl TestSurface {
    pub fn new(size: Size) -> Self {
        let len = size.width * size.height;
        let mut lines = vec![Glyph::space(); len as usize];

        Self { size, lines }
    }

    pub(crate) fn resize(&mut self, new_size: Size, glyph_map: &mut GlyphMap) {
        self.size = new_size;
        panic!("truncate and pop lines outside of the new size")
    }

    pub(crate) fn get(&self, x: usize, y: usize) -> Option<&Glyph> {
        let index = y * self.size.width as usize + x;
        self.lines.get(index)
    }
}

impl WidgetRenderer for TestSurface {
    fn draw_glyph(&mut self, mut glyph: Glyph, local_pos: Pos) {
        let index = self.size.height as i32 * local_pos.y + local_pos.x;
        let index = index as usize;
        std::mem::swap(&mut glyph, &mut self.lines[index]);
    }

    fn set_attributes(&mut self, attribs: &Attributes<'_>, local_pos: Pos) {
        let style = Style::from_cell_attribs(attribs);
        self.set_style(style, local_pos)
    }

    fn set_style(&mut self, style: Style, local_pos: Pos) {}

    fn size(&self) -> Size {
        self.size
    }
}
