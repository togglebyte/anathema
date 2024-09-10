use std::fmt::Display;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{Node, TreeValues};
use anathema_widgets::components::events::Event;
use anathema_widgets::paint::CellAttributes;
use anathema_widgets::{AttributeStorage, Element, WidgetKind, WidgetRenderer};

use crate::Backend;

pub struct TestBackend {
    pub surface: TestSurface,
    pub output: String,
}

impl TestBackend {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            surface: TestSurface::new(size),
            output: String::new(),
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

    fn resize(&mut self, _new_size: Size) {
        todo!()
    }

    fn paint<'bp>(
        &mut self,
        element: &mut Element<'bp>,
        children: &[Node],
        values: &mut TreeValues<WidgetKind<'bp>>,
        attribute_storage: &AttributeStorage<'bp>,
        ignore_floats: bool,
    ) {
        anathema_widgets::paint::paint(
            &mut self.surface,
            element,
            children,
            values,
            attribute_storage,
            ignore_floats,
        );
    }

    fn clear(&mut self) {
        self.surface.clear();
    }

    fn render(&mut self) {
        self.output = format!("{}", self.surface);
    }
}

pub struct TestSurface {
    size: Size,
    buffer: Vec<char>,
}

impl TestSurface {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        let buffer_size = size.width * size.height;
        Self {
            buffer: vec![' '; buffer_size],
            size,
        }
    }

    fn clear(&mut self) {
        self.buffer.fill_with(|| ' ');
    }
}

impl WidgetRenderer for TestSurface {
    fn draw_glyph(&mut self, c: char, local_pos: Pos) {
        let y_offset = local_pos.y as usize * self.size.width;
        let x_offset = local_pos.x as usize;
        let index = y_offset + x_offset;
        self.buffer[index] = c;
    }

    fn size(&self) -> Size {
        self.size
    }

    fn set_attributes(&mut self, _attribs: &dyn CellAttributes, _local_pos: Pos) {
        // NOTE: currently no attributes are stored on the test surface
    }
}

impl Display for TestSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let idx = y * self.size.width + x;
                write!(f, "{}", self.buffer[idx])?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
