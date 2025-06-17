use std::time::Duration;

use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::components::events::Event;
use anathema_widgets::paint::{Glyph, paint};
use anathema_widgets::{GlyphMap, PaintChildren};
use surface::TestSurface;

use crate::Backend;

mod events;
mod surface;

#[derive(Debug)]
/// This is used for testing
pub struct GlyphRef<'a> {
    inner: Option<&'a Glyph>,
}

impl<'a> GlyphRef<'a> {
    pub fn is_char(&self, rhs: char) -> bool {
        let Some(glyph) = self.inner else { return false };
        match glyph {
            Glyph::Single(lhs, _) => *lhs == rhs,
            Glyph::Cluster(_glyph_index, _) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct TestBackend {
    surface: TestSurface,
    events: events::Events,
}

impl TestBackend {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            surface: TestSurface::new(size),
            events: events::Events::new(),
        }
    }

    pub fn at(&self, x: usize, y: usize) -> GlyphRef<'_> {
        GlyphRef {
            inner: self.surface.get(x, y),
        }
    }

    pub fn line(&self, index: usize) -> String {
        let glyphs = self.surface.line(index);
        glyphs
            .iter()
            .filter_map(|g| match g {
                Glyph::Single(c, _) => Some(c),
                Glyph::Cluster(_, _) => None,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    pub fn events(&mut self) -> events::EventsMut<'_> {
        self.events.mut_ref()
    }
}

impl Backend for TestBackend {
    fn size(&self) -> Size {
        self.surface.size
    }

    fn next_event(&mut self, _timeout: Duration) -> Option<Event> {
        self.events.pop()
    }

    fn resize(&mut self, new_size: Size, glyph_map: &mut GlyphMap) {
        self.surface.resize(new_size, glyph_map);
    }

    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.surface.clear();
        paint(&mut self.surface, glyph_map, widgets, attribute_storage);
    }

    fn render(&mut self, _glyph_map: &mut GlyphMap) {
        // this does nothing here as everything written to the test buffer
    }

    fn clear(&mut self) {}
}
