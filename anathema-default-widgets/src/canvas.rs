use anathema::CommonVal;
use anathema_geometry::{LocalPos, Pos, Size};
use anathema_store::slab::Slab;
use anathema_store::smallmap::SmallMap;
use anathema_widgets::layout::text::StringSession;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{CellAttributes, PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Clone)]
pub enum CanvasAttrib {
    Str(String),
    Common(CommonVal<'static>),
}

impl<T: Into<CommonVal<'static>>> From<T> for CanvasAttrib {
    fn from(value: T) -> Self {
        Self::Common(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct CanvasAttribs(SmallMap<String, CanvasAttrib>);

impl CanvasAttribs {
    pub fn new() -> Self {
        Self(SmallMap::empty())
    }

    pub fn set_str(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<CanvasAttrib> {
        self.0.set(key.into(), CanvasAttrib::Str(value.into()))
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<CanvasAttrib>) -> Option<CanvasAttrib> {
        self.0.set(key.into(), value.into())
    }

    pub fn get(&self, key: &str) -> Option<&CanvasAttrib> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut CanvasAttrib> {
        self.0.get_mut(key)
    }
}

impl CellAttributes for CanvasAttribs {
    fn with_str(&self, key: &str, f: &mut dyn FnMut(&str)) {
        let Some(value) = self.get(key) else { return };
        match value {
            CanvasAttrib::Str(s) => f(s),
            CanvasAttrib::Common(CommonVal::Str(s)) => f(s),
            _ => {}
        }
    }

    fn get_i64(&self, key: &str) -> Option<i64> {
        let Some(CanvasAttrib::Common(CommonVal::Int(n))) = self.get(key) else { return None };
        Some(*n)
    }

    fn get_hex(&self, key: &str) -> Option<anathema::Hex> {
        let Some(CanvasAttrib::Common(CommonVal::Hex(n))) = self.get(key) else { return None };
        Some(*n)
    }

    fn get_bool(&self, key: &str) -> bool {
        matches!(self.get(key), Some(CanvasAttrib::Common(CommonVal::Bool(true))))
    }
}

#[derive(Debug, Default)]
enum Cell {
    #[default]
    Empty,
    Occupied(LocalPos, char, CanvasAttribs),
}

#[derive(Debug, Default, Clone, Copy)]
enum Entry {
    #[default]
    Vacant,
    Occupied(usize),
}

#[derive(Debug)]
struct Buffer {
    cells: Slab<usize, Cell>,
    positions: Box<[Entry]>,
    size: Size,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        Self {
            cells: Slab::empty(),
            positions: vec![Entry::Vacant; size.width * size.height].into_boxed_slice(),
            size,
        }
    }

    fn put(&mut self, c: char, attribs: CanvasAttribs, pos: LocalPos) {
        let cell_id = self.cells.next_id();

        let index = pos.to_index(self.size.width);
        if index >= self.positions.len() {
            return;
        }

        let cell = Cell::Occupied(pos, c, attribs);
        let mut entry = Entry::Occupied(cell_id);
        std::mem::swap(&mut self.positions[index], &mut entry);

        match entry {
            Entry::Vacant => {
                let new_cell_id = self.cells.insert(cell);
                assert_eq!(new_cell_id, cell_id);
            }
            Entry::Occupied(idx) => {
                self.cells.replace(idx, cell);
            }
        }
    }

    fn get(&mut self, pos: LocalPos) -> Option<&Cell> {
        let index = pos.to_index(self.size.width);
        match self.positions[index] {
            Entry::Occupied(idx) => self.cells.get(idx),
            Entry::Vacant => None,
        }
    }

    fn remove(&mut self, pos: LocalPos) {
        let index = pos.to_index(self.size.width);
        if index < self.positions.len() {
            let Entry::Occupied(idx) = std::mem::take(&mut self.positions[index]) else { return };
            self.cells.remove(idx);
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

    fn drain(&mut self) -> impl Iterator<Item = (LocalPos, char, CanvasAttribs)> + '_ {
        self.cells.consume().filter_map(|cell| match cell {
            Cell::Empty => None,
            Cell::Occupied(pos, c, attribs) => Some((pos, c, attribs)),
        })
    }

    fn iter(&self) -> impl Iterator<Item = (LocalPos, char, &CanvasAttribs)> + '_ {
        self.cells.iter().filter_map(|(_, cell)| match cell {
            Cell::Empty => None,
            Cell::Occupied(pos, c, attribs) => Some((*pos, *c, attribs)),
        })
    }
}

#[derive(Debug)]
pub struct Canvas {
    buffer: Buffer,
    pos: Pos,
}

impl Canvas {
    pub fn translate(&self, pos: Pos) -> LocalPos {
        let offset = pos - self.pos;
        LocalPos::new(offset.x as u16, offset.y as u16)
    }

    pub fn put(&mut self, c: char, attribs: CanvasAttribs, pos: LocalPos) {
        self.buffer.put(c, attribs, pos);
    }

    pub fn get(&mut self, pos: LocalPos) -> Option<(char, &CanvasAttribs)> {
        match self.buffer.get(pos)? {
            Cell::Occupied(_, c, attribs) => Some((*c, attribs)),
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
            buffer: Buffer::new((32, 32).into()),
            pos: Pos::ZERO,
        }
    }
}

impl Widget for Canvas {
    fn layout<'bp>(
        &mut self,
        _children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        let attribs = ctx.attribs.get(id);

        if let Some(width @ 0..=i64::MAX) = attribs.get_int("width") {
            constraints.set_max_width(width as usize);
        }

        if let Some(height @ 0..=i64::MAX) = attribs.get_int("height") {
            constraints.set_max_height(height as usize);
        }

        let size = constraints.max_size();

        if self.buffer.size != size {
            self.buffer = Buffer::copy_from(&mut self.buffer, size);
        }

        self.buffer.size
    }

    fn position<'bp>(
        &mut self,
        _children: PositionChildren<'_, '_, 'bp>,
        _id: WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.pos = ctx.pos;
    }

    fn paint<'bp>(
        &mut self,
        _children: PaintChildren<'_, '_, 'bp>,
        _id: WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
        _text: &mut StringSession<'_>,
    ) {
        for (pos, c, attribs) in self.buffer.iter() {
            ctx.place_glyph(c, attribs, pos);
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
