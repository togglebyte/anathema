use anathema_geometry::{LocalPos, Pos, Size};
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{Glyph, PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Style, Widget, WidgetId};
use anathema_widgets::error::Result;
use unicode_width::UnicodeWidthChar;

use crate::{HEIGHT, WIDTH};

#[derive(Debug, Default, Clone, Copy)]
enum Cell {
    #[default]
    Empty,
    Occupied(char, Style),
}

#[derive(Debug)]
struct Buffer {
    positions: Box<[Cell]>,
    size: Size,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        Self {
            positions: vec![Cell::Empty; size.width * size.height].into_boxed_slice(),
            size,
        }
    }

    fn put(&mut self, c: char, style: Style, pos: impl Into<LocalPos>) {
        let pos = pos.into();

        if pos.x as usize >= self.size.width || pos.y as usize >= self.size.height {
            return;
        }
        let index = pos.to_index(self.size.width);

        let mut cell = Cell::Occupied(c, style);
        std::mem::swap(&mut self.positions[index], &mut cell);
    }

    fn get(&self, pos: impl Into<LocalPos>) -> Option<&Cell> {
        let pos = pos.into();
        if pos.x as usize >= self.size.width || pos.y as usize >= self.size.height {
            return None;
        }

        let index = pos.to_index(self.size.width);
        match self.positions.get(index)? {
            cell @ Cell::Occupied(..) => Some(cell),
            Cell::Empty => None,
        }
    }

    fn get_mut(&mut self, pos: impl Into<LocalPos>) -> Option<&mut Cell> {
        let pos = pos.into();
        if pos.x as usize >= self.size.width || pos.y as usize >= self.size.height {
            return None;
        }

        let index = pos.to_index(self.size.width);
        match self.positions.get_mut(index)? {
            cell @ Cell::Occupied(..) => Some(cell),
            Cell::Empty => None,
        }
    }

    fn remove(&mut self, pos: impl Into<LocalPos>) {
        let pos = pos.into();
        if pos.x as usize >= self.size.width || pos.y as usize >= self.size.height {
            return;
        }

        let index = pos.to_index(self.size.width);
        if index < self.positions.len() {
            let mut cell = Cell::Empty;
            std::mem::swap(&mut self.positions[index], &mut cell);
        }
    }

    fn copy_from(other: &mut Buffer, size: Size) -> Self {
        let mut new_buffer = Buffer::new(size);

        for (pos, c, attrs) in other.drain() {
            if pos.x >= size.width as u16 || pos.y >= size.height as u16 {
                continue;
            }
            new_buffer.put(c, attrs, pos);
        }

        new_buffer
    }

    fn drain(&mut self) -> impl Iterator<Item = (LocalPos, char, Style)> + '_ {
        self.positions.iter_mut().enumerate().filter_map(|(index, cell)| {
            let mut old = Cell::Empty;
            std::mem::swap(&mut old, cell);
            //
            match old {
                Cell::Empty => None,
                Cell::Occupied(c, attribs) => {
                    let y = index / self.size.width;
                    let x = index % self.size.width;
                    let pos = LocalPos::new(x as u16, y as u16);
                    Some((pos, c, attribs))
                }
            }
        })
    }

    fn iter(&self) -> impl Iterator<Item = (LocalPos, char, &Style)> + '_ {
        self.positions.iter().enumerate().filter_map(|(index, cell)| {
            let x = index % self.size.width;
            let y = index / self.size.width;
            let pos = LocalPos::new(x as u16, y as u16);
            //
            match cell {
                Cell::Empty => None,
                Cell::Occupied(c, attribs) => Some((pos, *c, attribs)),
            }
        })
    }
}

#[derive(Debug)]
pub struct Canvas {
    buffer: Buffer,
    pos: Pos,
    is_dirty: bool,
}

impl Canvas {
    pub fn translate(&self, pos: Pos) -> LocalPos {
        let offset = pos - self.pos;
        LocalPos::new(offset.x as u16, offset.y as u16)
    }

    pub fn put(&mut self, c: char, style: Style, pos: impl Into<LocalPos>) {
        self.is_dirty = true;
        self.buffer.put(c, style, pos);
    }

    pub fn get(&mut self, pos: impl Into<LocalPos>) -> Option<(char, Style)> {
        match self.buffer.get(pos).copied()? {
            Cell::Occupied(c, style) => Some((c, style)),
            Cell::Empty => None,
        }
    }

    pub fn get_mut(&mut self, pos: impl Into<LocalPos>) -> Option<(&mut char, &mut Style)> {
        match self.buffer.get_mut(pos)? {
            Cell::Occupied(c, style) => {
                self.is_dirty = true;
                Some((c, style))
            }
            Cell::Empty => None,
        }
    }

    pub fn erase(&mut self, pos: impl Into<LocalPos>) {
        self.is_dirty = true;
        self.buffer.remove(pos)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            buffer: Buffer::new((32, 32).into()),
            pos: Pos::ZERO,
            is_dirty: true,
        }
    }
}

impl Widget for Canvas {
    fn layout<'bp>(
        &mut self,
        _: LayoutForEach<'_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let attribs = ctx.attribute_storage.get(id);

        if let Some(width) = attribs.get_as::<usize>(WIDTH) {
            constraints.set_max_width(width);
        }

        if let Some(height) = attribs.get_as::<usize>(HEIGHT) {
            constraints.set_max_height(height);
        }

        let size = constraints.max_size();

        if self.buffer.size != size {
            self.buffer = Buffer::copy_from(&mut self.buffer, size);
        }

        Ok(self.buffer.size)
    }

    fn position<'bp>(
        &mut self,
        _: PositionChildren<'_, 'bp>,
        _id: WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.pos = ctx.pos;
    }

    fn paint<'bp>(
        &mut self,
        _children: PaintChildren<'_, 'bp>,
        _id: WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        for (pos, c, style) in self.buffer.iter() {
            ctx.set_style(*style, pos);
            let glyph = Glyph::from_char(c, c.width().unwrap_or(0) as u8);
            ctx.place_glyph(glyph, pos);
        }
    }

    fn needs_reflow(&mut self) -> bool {
        let needs_reflow = self.is_dirty;
        self.is_dirty = false;
        needs_reflow
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::TestRunner;

    #[test]
    fn resize_canvas() {
        let expected = "
            ╔══╗
            ║  ║
            ║  ║
            ╚══╝
        ";
        TestRunner::new("canvas", (2, 2)).instance().render_assert(expected);
    }

    #[test]
    fn get_set_glyph() {
        let mut canvas = Canvas::default();
        canvas.put('a', Style::reset(), (0, 0));
        let (c, _) = canvas.get((0, 0)).unwrap();
        assert_eq!(c, 'a');
    }

    #[test]
    fn remove_glyph() {
        let mut canvas = Canvas::default();
        canvas.put('a', Style::reset(), (0, 0));
        assert!(canvas.get((0, 0)).is_some());
        canvas.erase((0, 0));
        assert!(canvas.get((0, 0)).is_none());
    }

    #[test]
    fn put_buffer_out_of_range() {
        let mut under_test = Canvas {
            buffer: Buffer::new(Size::new(1, 2)),
            ..Default::default()
        };

        under_test.put('x', Style::reset(), LocalPos::new(0, 0));
        under_test.put('x', Style::reset(), LocalPos::new(0, 1));
        under_test.put('o', Style::reset(), LocalPos::new(1, 0));

        for cell in under_test.buffer.positions {
            match cell {
                Cell::Empty => panic!("Should not be empty"),
                Cell::Occupied(c, _) => assert_eq!(c, 'x'),
            }
        }
    }

    #[test]
    fn get_buffer_out_of_range() {
        let mut under_test = Canvas {
            buffer: Buffer::new(Size::new(1, 2)),
            ..Default::default()
        };

        under_test.put('x', Style::reset(), LocalPos::new(0, 0));
        under_test.put('x', Style::reset(), LocalPos::new(0, 1));

        assert!(under_test.get(LocalPos::new(1, 0)).is_none());
    }

    #[test]
    fn get_mut_buffer_out_of_range() {
        let mut under_test = Canvas {
            buffer: Buffer::new(Size::new(1, 2)),
            ..Default::default()
        };

        under_test.put('x', Style::reset(), LocalPos::new(0, 0));
        under_test.put('x', Style::reset(), LocalPos::new(0, 1));

        assert!(under_test.get_mut(LocalPos::new(1, 0)).is_none());
    }

    #[test]
    fn remove_buffer_out_of_range() {
        let mut under_test = Canvas {
            buffer: Buffer::new(Size::new(1, 2)),
            ..Default::default()
        };

        under_test.put('x', Style::reset(), LocalPos::new(0, 0));
        under_test.put('x', Style::reset(), LocalPos::new(0, 1));
        under_test.erase(LocalPos::new(1, 0));

        for cell in under_test.buffer.positions {
            match cell {
                Cell::Empty => panic!("Should not be empty"),
                Cell::Occupied(c, _) => assert_eq!(c, 'x'),
            }
        }
    }
}
