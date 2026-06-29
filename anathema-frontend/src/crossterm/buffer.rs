use std::ops::{Index, Range};

use anathema_geometry::Region;

use crate::crossterm::{Cell, Style};

// -----------------------------------------------------------------------------
//   - Dirty Rows -
// -----------------------------------------------------------------------------

type Min = usize;
type Max = usize;

struct DirtyRows {
    inner: Vec<(Min, Max)>,
    first: usize,
    last: usize,
    dirty: bool,
    width: usize,
    height: usize,
}

impl DirtyRows {
    fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![(width, 0); height],
            dirty: false,
            first: 0,
            last: 0,
            width,
            height,
        }
    }

    fn insert(&mut self, row: usize, min: usize, max: usize) {
        self.dirty = true;
        self.last = self.last.max(row);
        self.first = self.first.min(row);
        let (current_min, current_max) = self.inner[row];
        self.inner[row] = (current_min.min(min), current_max.max(max));
    }

    fn reset(&mut self) {
        self.inner
            .iter_mut()
            .skip(self.first)
            .take(1 + self.last - self.first)
            .for_each(|row| *row = (self.width, 0));
        self.first = 0;
        self.last = 0;
        self.dirty = false;
    }

    fn iter(&self) -> impl Iterator<Item = (Min, Max)> {
        self.inner.iter().skip(self.first).take(self.last - self.first).copied()
    }

    fn rows(&self) -> impl Iterator<Item = (usize, Min, Max)> {
        self.inner
            .iter()
            .enumerate()
            .skip(self.first)
            .take(1 + self.last - self.first)
            .filter(|(_, (min, max))| max > min)
            .map(|(y, &(min, max))| (y, min, max))
    }

    fn clear(&mut self) {
        self.inner.iter_mut().for_each(|row| *row = (self.width, 0));
    }
}

impl Index<usize> for DirtyRows {
    type Output = (Min, Max);

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

// -----------------------------------------------------------------------------
//   - Buffer -
// -----------------------------------------------------------------------------

pub(super) struct Buffer {
    inner: Vec<Cell>,
    dirty_rows: DirtyRows,
}

impl Buffer {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![Cell::default(); width * height],
            dirty_rows: DirtyRows::new(width, height),
        }
    }

    pub(super) fn clear(&mut self) {
        self.inner.iter_mut().for_each(|cell| *cell = Cell::space());
        self.dirty_rows.clear();
    }

    pub(super) fn sync_buffers<'a>(&'a mut self, src: &Buffer, rows: &mut Vec<(usize, Diff)>) {
        for (y, (old, &new)) in std::iter::zip(&mut self.dirty_rows.inner, &src.dirty_rows.inner).enumerate() {
            match (*old, new) {
                // -----------------------------------------------------------------------------
                //   Empty, just skip
                // -----------------------------------------------------------------------------
                ((_, 0), (_, 0)) => continue,

                // -----------------------------------------------------------------------------
                //
                //   [||new||]
                //   Write
                // -----------------------------------------------------------------------------
                ((_, 0), (new_s, new_e)) => rows.push((y, Diff::Write(new_s..new_e))),

                // -----------------------------------------------------------------------------
                //   [||old||]
                //
                //   Erase
                // -----------------------------------------------------------------------------
                ((old_s, old_e), (_, 0)) => rows.push((y, Diff::ClearRow)),

                // -----------------------------------------------------------------------------
                //   [||old||]    [|||old|||]
                //     [||new||]    [||new||]
                //   Erase + Write
                // -----------------------------------------------------------------------------
                ((old_s, old_e), (new_s, new_e)) if old_s < new_s && old_e <= new_e => {
                    rows.push((y, Diff::ClearRange(old_s..new_s)));
                    rows.push((y, Diff::Write(new_s..new_e)));
                }

                // -----------------------------------------------------------------------------
                //   [|||old|||]
                //    [||new||]
                //   Erase + Write + Erase
                // -----------------------------------------------------------------------------
                ((old_s, old_e), (new_s, new_e)) if old_s < new_s && old_e > new_e => {
                    rows.push((y, Diff::ClearRange(old_s..new_s)));
                    rows.push((y, Diff::Write(new_s..new_e)));
                    rows.push((y, Diff::ClearRange(new_e..old_e)));
                }

                // -----------------------------------------------------------------------------
                //     [||old||]
                //   [||new||]
                //   Write + Erase
                // -----------------------------------------------------------------------------
                ((old_s, old_e), (new_s, new_e)) if old_s > new_s && old_e > new_e => {
                    rows.push((y, Diff::Write(new_s..new_e)));
                    rows.push((y, Diff::ClearRange(new_e..old_e)));
                }

                // -----------------------------------------------------------------------------
                //   [||old||]   [|old|]    [||old|]  [|old|]
                //   [||new||]  [||new||]  [||new||]  [||new||]
                //   Write
                // -----------------------------------------------------------------------------
                ((old_s, old_e), (new_s, new_e)) => rows.push((y, Diff::Write(new_s..new_e))),
            }

            let from = y * self.dirty_rows.width + new.0;
            let to = from + new.1 - new.0;
            self.inner[from..to].clone_from_slice(&src.inner[from..to]);

            *old = new;
        }
    }

    pub(super) fn begin_insert(&mut self, y: usize) -> RowInsert<'_> {
        RowInsert {
            start: self.dirty_rows.width,
            buffer: self,
            y,
            end: 0,
        }
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.dirty_rows.dirty
    }

    pub(crate) fn invalidate_region(&mut self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) {
        for y in start_y..end_y {
            let index = y * self.dirty_rows.width + start_x;
            let range = index..index + end_x;
            self.dirty_rows.insert(y, start_x, end_x);
            self.inner[range].iter_mut().for_each(|cell| *cell = Cell::space());
        }
    }

    pub(crate) fn clear_dirty_rows(&mut self) {
        self.dirty_rows.clear();
    }
}

impl Index<usize> for Buffer {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

// -----------------------------------------------------------------------------
//   - Row insert transaction -
// -----------------------------------------------------------------------------

pub(super) struct RowInsert<'a> {
    buffer: &'a mut Buffer,
    y: usize,
    start: usize,
    end: usize,
}

impl<'a> RowInsert<'a> {
    pub(super) fn write_state(&mut self, x: usize, state: super::State) {
        self.start = self.start.min(x);
        self.end = self.end.max(x + 1);
        let index = self.y * self.buffer.dirty_rows.width + x;
        self.buffer.inner[index].state = state;
    }

    pub(super) fn write_style(&mut self, range: Range<usize>, style: Style) {
        self.start = self.start.min(range.start);
        self.end = self.end.max(range.end);

        let from = self.y * self.buffer.dirty_rows.width + range.start;
        let to = from + range.end - range.start;
        let buffer = &mut self.buffer.inner[from..to];
        buffer.iter_mut().for_each(|cell| {
            cell.style.merge(style);
            cell.if_empty_make_space();
        });
    }
}

impl<'a> Drop for RowInsert<'a> {
    fn drop(&mut self) {
        self.buffer.dirty_rows.insert(self.y, self.start, self.end);
    }
}

// -----------------------------------------------------------------------------
//   - Diff -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(super) enum Diff {
    ClearRow,
    ClearRange(Range<usize>),
    ClearCell(usize),
    Write(Range<usize>),
}
