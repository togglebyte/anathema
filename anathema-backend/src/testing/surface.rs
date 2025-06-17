use anathema_geometry::{Pos, Size};
use anathema_value_resolver::Attributes;
use anathema_widgets::paint::Glyph;
use anathema_widgets::{GlyphMap, Style, WidgetRenderer};

#[derive(Debug)]
pub struct TestSurface {
    pub(super) size: Size,
    lines: Vec<Glyph>,
    prev_lines: Vec<Glyph>,
}

impl TestSurface {
    pub fn new(size: Size) -> Self {
        let len = size.width * size.height;
        let lines = vec![Glyph::space(); len as usize];
        Self {
            size,
            lines,
            prev_lines: vec![],
        }
    }

    pub(crate) fn resize(&mut self, new_size: Size, _glyph_map: &mut GlyphMap) {
        self.size = new_size;
        todo!("truncate and pop lines outside of the new size")
    }

    // Get from previous lines as the clear function has most likely
    // been called by the time this is relevant
    pub(crate) fn get(&self, x: usize, y: usize) -> Option<&Glyph> {
        let index = y * self.size.width as usize + x;
        self.prev_lines.get(index)
    }

    // Look at the old lines as the surface has been cleared
    // by the time we can inspect the lines
    pub(crate) fn line(&self, index: usize) -> &[Glyph] {
        let from = index * self.size.width as usize;
        let to = from + self.size.width as usize;
        &self.prev_lines[from..to]
    }

    pub(crate) fn clear(&mut self) {
        let len = self.size.width * self.size.height;
        let mut lines = vec![Glyph::space(); len as usize];
        std::mem::swap(&mut lines, &mut self.lines);
        std::mem::swap(&mut lines, &mut self.prev_lines);
    }
}

impl WidgetRenderer for TestSurface {
    fn draw_glyph(&mut self, mut glyph: Glyph, local_pos: Pos) {
        let index = self.size.width as i32 * local_pos.y + local_pos.x;
        let index = index as usize;
        std::mem::swap(&mut glyph, &mut self.lines[index]);
    }

    fn set_attributes(&mut self, attribs: &Attributes<'_>, local_pos: Pos) {
        let style = Style::from_cell_attribs(attribs);
        self.set_style(style, local_pos)
    }

    fn set_style(&mut self, _style: Style, _local_pos: Pos) {}

    fn size(&self) -> Size {
        self.size
    }
}
