use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_strings::HStrings;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::components::events::Event;
use anathema_widgets::paint::{CellAttributes, Glyph};
use anathema_widgets::{GlyphMap, WidgetRenderer};
use crossterm::cursor::position;
use crossterm::terminal::size;

use crate::Backend;

struct Surface {
    width: usize,
    position: Pos,
    draw: Vec<Pos>,
}

impl Surface {
    pub fn new() -> Self {
        let position = position().map(|(x, y)| Pos::new(x as i32, y as i32)).unwrap();
        let (width, _) = size().unwrap();
        let width = width as usize;
        Self {
            position,
            width,
            draw: vec![],
        }
    }

    fn drain(&mut self) {
        self.draw.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    }
}

impl WidgetRenderer for Surface {
    fn draw_glyph(&mut self, glyph: Glyph, local_pos: Pos) {
        todo!()
    }

    fn set_attributes(&mut self, attribs: &dyn CellAttributes, local_pos: Pos) {
        todo!()
    }

    fn size(&self) -> Size {
        panic!()
        // Size {
        //     width: self.width,
        //     height: usize::MAX,
        // }
    }
}

pub struct TuiScroll {
    surface: Surface,
}

impl Backend for TuiScroll {
    fn size(&self) -> Size {
        let s = size().expect("without the size there is no life");
        s.into()
    }

    fn next_event(&mut self, timeout: Duration) -> Option<Event> {
        None
    }

    fn resize(&mut self, _: Size, _: &mut GlyphMap) {}

    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: anathema_widgets::PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
        strings: &HStrings<'bp>,
        ignore_floats: bool,
    ) {
        anathema_widgets::paint::paint(
            &mut self.surface,
            glyph_map,
            widgets,
            attribute_storage,
            strings,
            ignore_floats,
        );
    }

    fn render(&mut self, glyph_map: &mut GlyphMap) {
        todo!()
    }

    fn clear(&mut self) {}
}
