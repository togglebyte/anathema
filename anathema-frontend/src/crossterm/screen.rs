use std::io::{stdout, Stdout, Write};
use std::ops::Index;

use anathema_geometry::{Pos, Region};
use anathema_store::scratch::ScratchBuffer;
use compact_str::CompactString;
use crossterm::style::{Attribute as CrossAttrib, Print, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{cursor, execute, QueueableCommand};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

use super::attributes::Attributes;
use super::buffer::{Buffer, Diff};
use super::State;
use crate::crossterm::{Cell, Style};
use crate::Frontend;

pub struct Screen<T> {
    front: Buffer,
    back: Buffer,
    width: usize,
    height: usize,
    output: T,
    clear_screen: bool,
    empty_line: String,
    scratch: ScratchBuffer<(usize, Diff)>,
}

impl Screen<Stdout> {
    pub fn new() -> Self {
        let (width, height) = crossterm::terminal::size().unwrap();
        Self::with_output(width as usize, height as usize, stdout())
    }
}

impl<T: Write> Screen<T> {
    pub fn with_output(width: usize, height: usize, output: T) -> Self {
        Self {
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            width,
            height,
            output,
            clear_screen: true,
            empty_line: " ".repeat(width),
            scratch: ScratchBuffer::empty(),
        }
    }

    pub fn enable_raw_mode(&self) {
        enable_raw_mode().unwrap();
    }

    pub fn disable_raw_mode(&self) {
        disable_raw_mode().unwrap();
    }

    pub fn clear(&mut self) {
        self.back.clear();
        self.front.clear();
        self.clear_screen = true;
    }

    fn style_region(&mut self, region: Region, style: Style) {
        let from_y = region.from.y as usize;
        let to_y = region.to.y as usize;
        let width = (region.to.x - region.from.x) as usize;
        let from_x = region.from.x as usize;
        let to_x = region.to.x as usize;
        for y in from_y..to_y {
            let mut insert = self.back.begin_insert(y);
            insert.write_style(from_x..to_x, style);
        }
    }

    pub fn render(&mut self) {
        if !self.back.is_dirty() && !self.clear_screen {
            return;
        }

        let _ = execute!(&mut self.output, BeginSynchronizedUpdate);

        // Reset cursor position
        self.output.queue(cursor::MoveTo(0, 0)).unwrap();

        if self.clear_screen {
            self.clear_screen = false;
            self.render_clear_screen();
            self.output.queue(cursor::MoveTo(0, 0)).unwrap();
        } else {
            self.render_partial();
        }

        self.front.clear_dirty_rows();
        self.back.clear_dirty_rows();
        self.output.flush();
        let _ = execute!(&mut self.output, EndSynchronizedUpdate);
    }

    fn render_partial(&mut self) {
        let mut buffer = vec![];
        self.front.sync_buffers(&self.back, &mut buffer);

        let mut last_y = 0;

        for (y, diff) in buffer {
            if y != last_y {
                self.output.queue(cursor::MoveToNextLine(1)).unwrap();
                last_y = y;
            }

            match diff {
                Diff::ClearRow => {
                    write_style(Style::reset(), &mut self.output);
                    self.output.queue(cursor::MoveTo(0, y as u16)).unwrap();
                    _ = self.output.queue(Print(&self.empty_line));
                }
                Diff::ClearRange(range) => {
                    write_style(Style::reset(), &mut self.output);
                    self.output.queue(cursor::MoveTo(range.start as u16, y as u16)).unwrap();
                    _ = self.output.queue(Print(&self.empty_line[range]));
                }
                Diff::ClearCell(_) => todo!(),
                Diff::Write(range) => {
                    let mut should_move = true;
                    let mut prev_style = None;

                    for x in range {
                        let index = y * self.width + x;
                        let cell = &self.front[index];

                        if should_move {
                            self.output.queue(cursor::MoveTo(x as u16, y as u16)).unwrap();
                            should_move = false;
                        }

                        if prev_style != Some(cell.style) {
                            prev_style = Some(cell.style);
                            write_style(cell.style, &mut self.output);
                        }

                        // write the character
                        match &cell.state {
                            super::State::Empty | super::State::Continuation => should_move = true,
                            super::State::Char(c) => _ = self.output.queue(Print(c)),
                            super::State::Cluster(cluster) => _ = self.output.queue(Print(cluster)),
                        }
                    }
                }
            }
        }
    }

    fn render_clear_screen(&mut self) {
        write_style(Style::reset(), &mut self.output);

        for y in 0..self.height {
            self.output.queue(Print(&self.empty_line));
            if y < self.height {
                self.output.queue(cursor::MoveToNextLine(1)).unwrap();
            }
        }
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
    fn apply_brush_to_region(&mut self, brush: &dyn crate::Brush, region: Region) {
        let style = Style::from(brush);
        self.style_region(region, style);
    }

    fn set_text(&mut self, text: &str, pos: Pos) {
        let graphemes = text.graphemes(true);
        let y = pos.y as usize;
        let mut x = pos.x as usize;
        let mut insertion = self.back.begin_insert(y);

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
            insertion.write_state(x, state);
            if width > 1 {
                insertion.write_state(x + 1, State::Continuation);
            }
            x += width;
        }
    }

    fn invalidate_region(&mut self, region: Region) {
        let start_x = region.from.x as usize;
        let end_x = region.to.x as usize;
        let start_y = region.from.y as usize;
        let end_y = region.to.y as usize;
        self.back.invalidate_region(start_x, start_y, end_x, end_y);
    }
}
