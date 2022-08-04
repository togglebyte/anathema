use crate::display::{Buffer, ScreenPos, Size, Style};

use super::LocalPos;
use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};
use crate::widgets::{fields, Attributes};

#[derive(Debug)]
enum CanvasState {
    Unsized,
    Sized(Buffer),
}

/// A canvas widget is used to place characters with a given [`Style`] at a given position ([`LocalPos`]).
/// ```
/// use anathema::widgets::{Canvas, LocalPos};
/// use anathema::display::Style;
/// let mut canvas = Canvas::new(8, 5);
/// let style = Style::new();
/// canvas.put('H', style, LocalPos::new(2, 2));
/// canvas.put('i', style, LocalPos::new(3, 2));
/// ```
/// output:
///
/// ```text
/// ┌──────┐
/// │      │
/// │ Hi   │
/// │      │
/// └──────┘
/// ```
///
/// An unsized `Canvas` will resize it self to fill the constraints.
/// If a canvas is resized it will truncate what is drawn.
#[derive(Debug)]
pub struct Canvas {
    needs_layout: bool,
    needs_paint: bool,
    state: CanvasState,
}

impl Canvas {
    /// Create a new instance of a `Canvas`.
    /// If no width or height is provided the canvas will fill the available space
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        let width = width.into();
        let height = height.into();

        let state = match (width, height) {
            (Some(w), Some(h)) => CanvasState::Sized(Buffer::new(Size::new(w, h))),
            _ => CanvasState::Unsized,
        };

        Self { needs_layout: true, needs_paint: true, state }
    }

    /// Put a character somewhere on the canvas
    pub fn put(&mut self, c: char, style: Style, pos: LocalPos) {
        let buffer = match self.state {
            CanvasState::Sized(ref mut buffer) => buffer,
            CanvasState::Unsized => return,
        };

        let size = buffer.size();
        if pos.x >= size.width || pos.y >= size.height {
            return;
        }

        let pos = ScreenPos::new(pos.x as u16, pos.y as u16);
        buffer.put_char(c, style, pos);
        self.needs_paint = true;
    }

    /// Clear a character on the canvas
    pub fn clear(&mut self, pos: LocalPos) {
        let buffer = match self.state {
            CanvasState::Sized(ref mut buffer) => buffer,
            CanvasState::Unsized => return,
        };

        let size = buffer.size();
        if pos.x >= size.width || pos.y >= size.height {
            return;
        }

        let pos = ScreenPos::new(pos.x as u16, pos.y as u16);
        buffer.empty(pos);
    }
}

impl Widget for Canvas {
    fn kind(&self) -> &'static str {
        "Canvas"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn needs_layout(&mut self) -> bool {
        // self.needs_layout
        true
    }

    fn needs_paint(&self) -> bool {
        // self.needs_paint
        true
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        self.needs_layout = false;

        match self.state {
            CanvasState::Unsized => {
                let size = Size::new(ctx.constraints.max_width, ctx.constraints.max_height);
                let buffer = Buffer::new(size);
                self.state = CanvasState::Sized(buffer);
                size
            }
            CanvasState::Sized(ref buffer) => buffer.size(),
        }
    }

    fn position(&mut self, _: PositionCtx) {}

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        let buffer = match self.state {
            CanvasState::Unsized => return,
            CanvasState::Sized(ref mut buffer) => buffer,
        };

        self.needs_paint = false;
        for (y, line) in buffer.rows().enumerate() {
            for (x, cell) in line.enumerate() {
                if let Some((c, style)) = cell {
                    let pos = LocalPos::new(x, y);
                    ctx.put(c, style, pos);
                }
            }
        }
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        vec![]
    }

    fn add_child(&mut self, _: WidgetContainer) {}

    fn remove_child(&mut self, _: &NodeId) -> Option<WidgetContainer> {
        None
    }

    fn update(&mut self, attributes: Attributes) {
        let buf = match &mut self.state {
            CanvasState::Unsized => return,
            CanvasState::Sized(buf) => buf,
        };

        let mut size = buf.size();

        if attributes.has(fields::WIDTH) {
            attributes.width().map(|width| size.width = width);
        }

        if attributes.has(fields::HEIGHT) {
            attributes.height().map(|height| size.height = height);
        }

        if buf.size() != size {
            buf.resize(size);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::testing::test_widget;
    use crate::widgets::{Border, BorderStyle, LocalPos, Sides};

    fn test_canvas(positions: Vec<LocalPos>, expected: &str) {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        let mut canvas = Canvas::new(10, 10);
        for pos in positions {
            canvas.put('x', Style::reset(), pos);
        }
        border.add_child(canvas.into_container(NodeId::auto()));
        test_widget(border, expected);
    }

    #[test]
    fn canvas() {
        test_canvas(
            vec![LocalPos::ZERO, LocalPos::new(7, 1), LocalPos::new(3, 2)],
            r#"
            ┌────────┐
            │x       │
            │       x│
            │   x    │
            └────────┘
            "#,
        );
    }
}
