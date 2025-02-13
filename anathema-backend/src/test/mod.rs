use std::fmt::Display;

use anathema_geometry::{Pos, Size};
use anathema_store::slab::SlabIndex;
use anathema_value_resolver::{AttributeStorage, Attributes};
use anathema_widgets::components::events::Event;
use anathema_widgets::paint::{Glyph, GlyphMap};
use anathema_widgets::{PaintChildren, WidgetRenderer};

use crate::Backend;

pub struct TestBackend {
    pub surface: TestSurface,
    pub glyph_map: GlyphMap,
    pub output: String,
}

impl TestBackend {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            surface: TestSurface::new(size),
            output: String::new(),
            glyph_map: GlyphMap::empty(),
        }
    }
}

impl Backend for TestBackend {
    fn size(&self) -> Size {
        self.surface.size
    }

    fn next_event(&mut self, _timeout: std::time::Duration) -> Option<Event> {
        None
    }

    fn resize(&mut self, _new_size: Size, _: &mut GlyphMap) {
        todo!()
    }

    fn paint<'bp>(
        &mut self,
        _: &mut GlyphMap,
        widgets: PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        anathema_widgets::paint::paint(&mut self.surface, &mut self.glyph_map, widgets, attribute_storage);
    }

    fn clear(&mut self) {
        self.surface.clear();
    }

    fn render(&mut self, _: &mut GlyphMap) {
        self.output = format!("{}", self.surface);
    }
}

pub struct TestSurface {
    size: Size,
    buffer: Vec<Glyph>,
}

impl TestSurface {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        let buffer_size = size.width * size.height;
        Self {
            buffer: vec![Glyph::space(); buffer_size],
            size,
        }
    }

    fn clear(&mut self) {
        self.buffer.fill_with(Glyph::space);
    }
}

impl WidgetRenderer for TestSurface {
    fn draw_glyph(&mut self, c: Glyph, local_pos: Pos) {
        let y_offset = local_pos.y as usize * self.size.width;
        let x_offset = local_pos.x as usize;
        let index = y_offset + x_offset;
        self.buffer[index] = c;
    }

    fn size(&self) -> Size {
        self.size
    }

    fn set_attributes(&mut self, _attribs: &Attributes<'_>, _local_pos: Pos) {
        // NOTE: currently no attributes are stored on the test surface
    }

    fn set_style(&mut self, style: anathema_widgets::Style, local_pos: Pos) {
        // This test surface should probably be removed
    }
}

impl Display for TestSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let idx = y * self.size.width + x;
                match self.buffer[idx] {
                    Glyph::Single(c, _) => write!(f, "{}", c)?,
                    Glyph::Cluster(i, _) => write!(f, "({})", i.as_usize())?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
