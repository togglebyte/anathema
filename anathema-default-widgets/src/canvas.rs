use anathema_backend::tui::{Buffer, Style};
use anathema_geometry::{LocalPos, Pos, Size};
use anathema_store::slab::Slab;
use anathema_widgets::layout::text::StringSession;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Default, Copy, Clone)]
enum Cell {
    #[default]
    Empty,
    Occupied(LocalPos, char, Style),
}

#[derive(Debug, Default)]
enum Entry {
    #[default]
    Vacant,
    Occupied(usize),
}

#[derive(Debug)]
struct Buffer2 {
    inner: Slab<usize, Cell>,
    positions: Vec<Entry>,
    size: Size,
}

impl Buffer2 {
    pub fn new(size: Size) -> Self {
        Self {
            inner: Slab::empty(),
            positions: Vec::with_capacity(size.width * size.height),
            size,
        }
    }

    fn put(&mut self, c: char, style: Style, pos: LocalPos) {
        let data = self.inner.next_id();

        let index = pos.to_index(self.size.width);
        if index < self.positions.len() {
            let cell = Cell::Occupied(pos, c, style);
            let mut entry = Entry::Occupied(data);
            std::mem::swap(&mut self.positions[index], &mut entry);

            match entry {
                Entry::Vacant => {
                    self.inner.insert(cell);
                }
                Entry::Occupied(idx) => {
                    self.inner.replace(idx, cell);
                }
            }
        }
    }

    fn get(&mut self, pos: LocalPos) -> Option<Cell> {
        let index = pos.to_index(self.size.width);
        match self.positions[index] {
            Entry::Occupied(idx) => self.inner.get(idx).copied(),
            Entry::Vacant => None,
        }
    }

    fn remove(&mut self, pos: LocalPos) {
        let index = pos.to_index(self.size.width);
        if index < self.positions.len() {
            let Entry::Occupied(idx) = std::mem::take(&mut self.positions[index]) else { return };
            self.inner.remove(idx);
        }
    }

    //     fn copy_from(other: &mut Buffer2, size: Size) -> Self {
    //         let mut inner = vec![Cell::Empty; size.width * size.height];

    //         // Copy from other to self
    //         for i in 0..other.inner.len() {
    //             let y = i / other.size.width;
    //             let x = i - y * other.size.width;

    //             if x >= size.width {
    //                 continue;
    //             }

    //             if y >= size.height {
    //                 break;
    //             }

    //             let j = y * size.width + x;

    //             std::mem::swap(
    //                 &mut inner[j],
    //                 &mut other.inner[i]
    //             );
    //         }

    //         Buffer2 { inner, size }
    //     }

    fn iter(&self) -> impl Iterator<Item = (LocalPos, char, Style)> + '_ {
        self.inner.iter().filter_map(|(_, cell)| match cell {
            Cell::Empty => None,
            Cell::Occupied(pos, c, style) => Some((*pos, *c, *style)),
        })
    }
}

#[derive(Debug)]
pub struct Canvas {
    constraints: Constraints,
    buffer: Buffer2,
    pos: Pos,
}

impl Canvas {
    pub fn translate(&self, pos: Pos) -> LocalPos {
        let offset = pos - self.pos;
        LocalPos::new(offset.x as u16, offset.y as u16)
    }

    pub fn put(&mut self, c: char, style: Style, pos: LocalPos) {
        self.buffer.put(c, style, pos);
    }

    pub fn get(&mut self, pos: LocalPos) -> Option<(char, Style)> {
        match self.buffer.get(pos)? {
            Cell::Occupied(_, c, style) => Some((c, style)),
            Cell::Empty => None,
        }
    }

    pub fn erase(&mut self, pos: LocalPos) {
        self.buffer.remove(pos)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            constraints: Constraints::unbounded(),
            buffer: Buffer2::new((32, 32).into()),
            pos: Pos::ZERO,
        }
    }
}

impl Widget for Canvas {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        let size = constraints.max_size();

        if self.buffer.size != size {
            panic!("resize buffer")
            // self.buffer.resize(size);
        }

        self.buffer.size
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.pos = ctx.pos;
    }

    fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
        text: &mut StringSession<'_>,
    ) {
        for (pos, c, style) in self.buffer.iter() {
            // ctx.place_glyph(c, attribs, pos);
        }
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
}
