use anathema_geometry::{LocalPos, Pos, Size};
use anathema_widgets::layout::text::StringSession;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Default)]
struct Buffer {
    inner: Vec<Cell>,
    size: Size,
}

impl Buffer {
    fn put(&mut self, c: char, pos: LocalPos) {
        let index = pos.y as usize * self.size.width + pos.x as usize;
        if index < self.inner.len() {
            self.inner[index] = Cell::Occupied(c);
        }
    }

    fn get(&mut self, pos: LocalPos) -> Option<char> {
        let index = pos.y as usize * self.size.width + pos.x as usize;
        match self.inner[index] {
            Cell::Occupied(c) => Some(c),
            Cell::Empty => None,
        }
    }

    fn remove(&mut self, pos: LocalPos) {
        let index = pos.y as usize * self.size.width + pos.x as usize;
        if index < self.inner.len() {
            self.inner[index] = Cell::Empty;
        }
    }

    fn copy_from(other: &mut Buffer, size: Size) -> Self {
        let mut inner = vec![Cell::Empty; size.width * size.height];

        // Copy from other to self
        for i in 0..other.inner.len() {
            let y = i / other.size.width;
            let x = i - y * other.size.width;

            if x > size.width {
                continue;
            }

            if y > size.height {
                break;
            }

            let j = y * size.width + x;

            std::mem::swap(
                &mut inner[j], 
                &mut other.inner[i]
            );
        }

        Buffer { inner, size }
    }

    fn iter(&self) -> impl Iterator<Item = (char, LocalPos)> + '_ {
        self.inner
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(i, cell)| match cell {
                Cell::Empty => None,
                Cell::Occupied(c) => {
                    let y = i / self.size.width;
                    let x = i - y * self.size.width;
                    let pos = LocalPos::new(x as u16, y as u16);
                    Some((c, pos))
                }
            })
    }
}

#[derive(Debug, Default, Copy, Clone)]
enum Cell {
    #[default]
    Empty,
    Occupied(char),
}

#[derive(Debug)]
pub struct Canvas {
    constraints: Constraints,
    buffer: Buffer,
    pos: Pos,
}

impl Canvas {
    pub fn translate(&self, pos: Pos) -> LocalPos {
        let offset = pos - self.pos;
        LocalPos::new(offset.x as u16, offset.y as u16)
    }

    pub fn put(&mut self, c: char, pos: LocalPos) {
        self.buffer.put(c, pos);
    }

    pub fn get(&mut self, pos: LocalPos) -> Option<char> {
        self.buffer.get(pos)
    }

    pub fn erase(&mut self, pos: LocalPos) {
        self.buffer.remove(pos)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            constraints: Constraints::unbounded(),
            buffer: Buffer {
                inner: vec![Cell::Empty; 1024],
                size: Size::new(32, 32),
            },
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
            self.buffer = Buffer::copy_from(&mut self.buffer, size);
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
        let attribs = attribute_storage.get(id);
        for (c, pos) in self.buffer.iter() {
            ctx.place_glyph(c, attribs, pos);
        }
    }
}
