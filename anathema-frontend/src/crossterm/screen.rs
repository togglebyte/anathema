use std::io::{stdout, Stdout, Write};
use std::ops::Index;

use anathema_geometry::{Pos, Region};
use compact_str::CompactString;
use crossterm::style::{Attribute as CrossAttrib, Print, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{cursor, execute, QueueableCommand};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use super::attributes::Attributes;
use super::buffer::Buffer;
use super::State;
use crate::crossterm::Style;
use crate::Frontend;

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
            .take(self.last - self.first)
            .for_each(|row| *row = (self.width, 0));
    }

    fn is_empty(&self) -> bool {
        self.first == 0 && self.last == 0
    }

    fn iter(&self) -> impl Iterator<Item = (Min, Max)> {
        self.inner.iter().skip(self.first).take(self.last - self.first).copied()
    }

    fn rows(&self) -> impl Iterator<Item = (usize, Min, Max)> {
        self.inner
            .iter()
            .enumerate()
            .skip(self.first)
            .take(self.last - self.first)
            .filter(|(_, (min, max))| max > min)
            .map(|(y, &(min, max))| (y, min, max))
    }
}

impl Index<usize> for DirtyRows {
    type Output = (Min, Max);

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

pub struct Screen<T> {
    front: Buffer,
    back: Buffer,
    dirty_rows: DirtyRows,
    width: usize,
    height: usize,
    output: T,
}

impl Screen<Stdout> {
    pub fn new(width: usize, height: usize) -> Self {
        Self::with_output(width, height, stdout())
    }
}

impl<T: Write> Screen<T> {
    pub fn with_output(width: usize, height: usize, output: T) -> Self {
        Self {
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            dirty_rows: DirtyRows::new(width, height),
            width,
            height,
            output,
        }
    }

    fn dirty_percentage(&self) -> f32 {
        let total_cells = (self.dirty_rows.width * self.dirty_rows.height) as f32;
        let mut dirty_cells = 0;
        for (min, max) in self.dirty_rows.iter() {
            dirty_cells += max.saturating_sub(min);
        }

        dirty_cells as f32 / total_cells
    }

    // Write a row from the back buffer into the front buffer
    fn write_row(&mut self, y: usize) {
        let (start, end) = self.dirty_rows[y];
        self.front.copy_range(start..end, &self.back);
    }

    pub fn style_region(&mut self, region: Region, style: Style) {
        let from_y = region.from.y as usize;
        let to_y = region.to.y as usize;
        let width = (region.to.x - region.from.x) as usize;
        let x = region.from.x as usize;
        for y in from_y..to_y {
            let index = y * self.width + x;
            let cells = self.back.slice_mut(index..index + width);
            cells.iter_mut().for_each(|cell| cell.style.merge(style));
        }
    }

    pub fn render(&mut self) {
        if self.dirty_rows.is_empty() {
            return;
        }

        let _ = execute!(&mut self.output, BeginSynchronizedUpdate);
        if self.dirty_percentage() >= 0.6 {
            self.render_full();
        } else {
            self.render_partial();
        }

        // let _ = self.screen.render(&mut self.output, glyph_map);

        let _ = execute!(&mut self.output, EndSynchronizedUpdate);
        self.output.flush();

        self.dirty_rows.reset();
    }

    fn render_partial(&mut self) {
        let mut prev_style = None;
        let mut should_move = true;

        for (y, mut x, max) in self.dirty_rows.rows() {
            let move_to = (x, y);
            self.output.queue(cursor::MoveTo(x as u16, y as u16)).unwrap();

            // move to move_to
            let start = y * self.width + x;
            let end = start + max - x;
            let cells = self.front.cells(start..end);
            for cell in cells {

                if should_move {
                    self.output.queue(cursor::MoveTo(x as u16, y as u16)).unwrap();
                    should_move = false;
                }

                if prev_style != Some(cell.style) {
                    // write the style
                    write_style(cell.style, &mut self.output);
                    _ = prev_style.replace(cell.style);
                }

                // write the character
                match &cell.state {
                    super::State::Empty => should_move = true,
                    super::State::Continuation => (),
                    super::State::Char(c) => _ = self.output.queue(Print(c)),
                    super::State::Cluster(cluster) => _ = self.output.queue(Print(cluster)),
                }

                x += cell.state.width();
            }
        }
    }

    fn render_full(&mut self) {
        // let mut prev_style = None;
        // for row in self.front.rows() {
        //     // for cell in cells {
        //     //     if prev_style != Some(cell.style) {
        //     //         // write the style
        //     //         prev_style.replace(cell.style);
        //     //     }

        //     //     // write the character
        //     // }
        // }
    }
}

fn write_style(style: Style, output: &mut impl Write) {
    if let Some(fg) = style.fg {
        output.queue(SetForegroundColor(fg.into()));
    }

    if let Some(bg) = style.bg {
        output.queue(SetBackgroundColor(bg.into()));
    }

    // Dim and bold are a special case, as they are both
    // reset through `NormalIntensity` (22).
    // This means the reset has to happen before setting
    // bold or dim
    if !style.attributes.contains(Attributes::BOLD | Attributes::DIM) {
        output.queue(SetAttribute(CrossAttrib::NormalIntensity));
    }

    if style.attributes.contains(Attributes::NORMAL) {
        output.queue(SetAttribute(CrossAttrib::NormalIntensity));
    } else if style.attributes.contains(Attributes::BOLD) {
        output.queue(SetAttribute(CrossAttrib::Bold));
    }

    if style.attributes.contains(Attributes::DIM) {
        output.queue(SetAttribute(CrossAttrib::Dim));
    }

    macro_rules! check {
        ($inc:expr, $exc:expr) => {
            style.attributes.contains($inc) && !style.attributes.contains($exc)
        };
    }

    // Italic
    if check!(Attributes::ITALIC, Attributes::NOT_ITALIC) {
        output.queue(SetAttribute(CrossAttrib::Italic));
    } else {
        output.queue(SetAttribute(CrossAttrib::NoItalic));
    }

    // Underlined
    if check!(Attributes::UNDERLINED, Attributes::NOT_UNDERLINED) {
        output.queue(SetAttribute(CrossAttrib::Underlined));
    } else {
        output.queue(SetAttribute(CrossAttrib::NoUnderline));
    }

    if check!(Attributes::OVERLINED, Attributes::NOT_OVERLINED) {
        output.queue(SetAttribute(CrossAttrib::OverLined));
    } else {
        output.queue(SetAttribute(CrossAttrib::NotOverLined));
    }

    if check!(Attributes::CROSSED_OUT, Attributes::NOT_CROSSED_OUT) {
        output.queue(SetAttribute(CrossAttrib::CrossedOut));
    } else {
        output.queue(SetAttribute(CrossAttrib::NotCrossedOut));
    }

    if check!(Attributes::REVERSED, Attributes::NOT_REVERSED) {
        output.queue(SetAttribute(CrossAttrib::Reverse));
    } else {
        output.queue(SetAttribute(CrossAttrib::NoReverse));
    }

    // Ok(())
}

impl<T: Write> Frontend for Screen<T> {
    fn apply_brush_to_region(&mut self, region: Region, brush: &dyn crate::Brush) {
        let style = Style::from(brush);
        self.style_region(region, style);
    }

    fn set_text(&mut self, pos: Pos, text: &str) {
        let graphemes = text.graphemes(true);
        let y = pos.y as usize;
        let mut x = pos.x as usize;

        for grapheme in graphemes {
            let mut chars = grapheme.chars();
            let Some(c) = chars.next() else { continue };

            // Ignore any zero width characters
            let width = match c.width() {
                Some(0) | None => continue,
                Some(w) => w,
            };

            let state = match chars.next() {
                Some(_) => State::Cluster(CompactString::new(grapheme)),
                None => State::Char(c),
            };

            // write
            let index = y * self.width + x;
            self.back.write(index, state);

            if width > 1 {
                self.back.write(index + 1, State::Continuation);
            }

            x += width;
        }
    }
}
