use anathema_geometry::Size;
use anathema_store::tree::ValueId;
use anathema_widgets::layout::{Entry, TextIndex, TextSession};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::ProcessResult;

fn is_word_boundary(c: char) -> bool {
    c.is_whitespace() || c == '-'
}

#[derive(Debug, Default)]
pub struct Word {
    width: usize,
    pos: usize,
    end_pos: usize,
}

impl Word {
    fn transfer_word_to_buffer(&mut self, word: &str, max: usize, used_width: &mut usize) {
        let mut chars = word
            .char_indices()
            .map(|(i, c)| (i + c.len_utf8(), c.width().unwrap_or(0)));

        let mut end = 0;
        loop {
            let Some((next_char_pos, char_width)) = chars.next() else { break };
            if *used_width + char_width <= max {
                *used_width += char_width;
                self.width -= char_width;
                end = next_char_pos;
            } else {
                break;
            }
        }
        self.end_pos += end;
    }
}

#[derive(Debug)]
pub struct WordWrapLayout<'a> {
    current: Word,
    session: TextSession<'a>,
    index: TextIndex,
    used_width: usize,
    max: Size,
    size: Size,
}

impl<'a> WordWrapLayout<'a> {
    pub fn new(max: impl Into<Size>, session: TextSession<'a>, index: TextIndex) -> Self {
        let max = max.into();
        Self {
            current: Word::default(),
            session,
            index,
            used_width: 0,
            max,
            size: Size { width: 0, height: 1 },
        }
    }

    // if the input is larger (width > remaining width) than available space then split
    //
    // A) if the current width is larger than zero
    //    insert newline before the new word
    //
    // if the input is larger than available space then split
    //
    // B) if the word has a trailing whitespace that pushes it over the
    //    size constraint then remove that whitespace
    //
    // if the input is larger than available space then split
    //
    // C) if the word is wider than max_width then split the word
    //
    // if the input ends in a newline char
    //
    // D) reset line width
    fn process_word(&mut self) -> ProcessResult {
        if self.max.width == 0 || self.max.height == 0 {
            return ProcessResult::Done;
        }

        loop {
            let available = self.max.width - self.used_width;

            // A) if the current width is larger than zero
            //    insert newline before the new word
            if self.current.width > available && self.used_width != 0 {
                if self.size.height >= self.max.height {
                    return ProcessResult::Done;
                }

                self.newline();

                continue;
            }

            // B) if the word has a trailing whitespace that pushes it over the
            //    size constraint then remove that whitespace
            let last_char = self
                .session
                .bytes
                .word(self.index.bytes, self.current.pos)
                .chars()
                .next_back();
            if self.current.width > available {
                if let Some(c) = last_char {
                    if c.is_whitespace() {
                        let width = c.width().unwrap_or(0);
                        if self.current.width - width <= available {
                            self.session.bytes.tail_drain(c.len_utf8());
                            self.current.width -= width;
                        }
                    }
                }
            }

            // C) if the word is wider than max_width then split the word
            if self.current.width > self.max.width && self.used_width == 0 {
                let word = self.session.bytes.word(self.index.bytes, self.current.pos);
                self.current
                    .transfer_word_to_buffer(word, self.max.width, &mut self.used_width);
                self.store_word();
                continue;
            }

            break;
        }

        let ends_with_newline = self.session.bytes.ends_with_newline();
        // If the word ends with a newline character, then
        // pop the character, and insert a NewLine **after**
        // the word has been processed.
        if ends_with_newline {
            self.session.bytes.pop();
        }

        let word = self.session.bytes.word(self.index.bytes, self.current.pos);
        if !word.is_empty() {
            self.current
                .transfer_word_to_buffer(word, self.max.width, &mut self.used_width);
            self.store_word();
        }

        self.update_line_width(self.used_width);
        self.size.width = self.size.width.max(self.used_width);

        if ends_with_newline {
            self.newline();
        }

        ProcessResult::Continue
    }

    fn store_word(&mut self) {
        self.session
            .layout
            .push(Entry::Str(self.current.pos, self.current.end_pos));
        self.current.pos = self.current.end_pos;
    }

    fn newline(&mut self) {
        self.update_line_width(self.used_width);
        self.session.layout.push(Entry::Newline);
        self.session.layout.push(Entry::LineWidth(0));
        self.size.width = self.size.width.max(self.used_width);
        self.used_width = 0;
        self.size.height += 1;
    }

    pub(crate) fn set_style(&mut self, style: ValueId) {
        self.session.layout.push(Entry::SetStyle(style));
    }

    pub fn process(&mut self, s: &str) -> ProcessResult {
        for word in s.split_inclusive(is_word_boundary) {
            self.session.bytes.extend(word.bytes());
            self.current.width += word.width();

            if word.ends_with(is_word_boundary) {
                match self.process_word() {
                    done @ ProcessResult::Done => return done,
                    ProcessResult::Continue => continue,
                }
            }
        }

        ProcessResult::Continue
    }

    pub fn finish(&mut self) {
        self.process_word();
    }

    pub fn size(mut self) -> Size {
        if self.size.width == 0 {
            self.size = Size::ZERO;
        }
        self.size
    }

    fn update_line_width(&mut self, used_width: usize) {
        let layout = self.session.layout.slice_mut(self.index.layout);
        if let Some(width) = layout.iter_mut().rev().find_map(|e| match e {
            Entry::LineWidth(width) => Some(width),
            _ => None,
        }) {
            *width = used_width as u32;
        }
    }
}

#[cfg(test)]
mod test {

    use anathema_widgets::layout::{IterEntry, TextBuffer};

    use crate::layout::text::testing::layout;
    use crate::layout::text::{Lines, TextLayout};
    use crate::Wrap;

    #[test]
    fn inserts() {
        //           Max 5
        //           1    2    3   4    5    6
        //           1    2  3 4   5    6    7
        // input = ["a", " ", "b", c", "d", "e"];
        //                   ^ |        |    |
        //  NL here ---------+ |        |    |
        //                     |        |    |
        //           Red-------+        |    |
        //                       Green -+    |
        //                                   Bold

        let inputs: &[(&[&str], &str)] = &[
            (&[" 12", "345　12", "345 "], " \n12345\n12345"),
            (&[" 🐇🐇🐇", "🐇🐇 12", "345 "], " \n🐇🐇\n🐇🐇\n🐇 \n12345"),
            (&["1", "23", "45 12", "345 "], "12345\n12345"),
            (&["12345 abcde "], "12345\nabcde"),
            (&["onereallylongword"], "onere\nallyl\nongwo\nrd"),
            (&["ahello do the"], "ahell\no do \nthe"),
            (&["hello do the"], "hello\ndo \nthe"),
        ];

        let mut text_buffer = TextBuffer::empty();

        for input in inputs {
            let size = (5, 10);

            let mut session = text_buffer.new_session();
            let key = session.new_key();
            let mut layout = TextLayout::new(size, Wrap::Normal, session, key);

            // Layout
            for part in input.0 {
                layout.process(part);
            }

            layout.finish();
            layout.size();

            // Read
            let session = text_buffer.new_session();
            let lines = Lines::new(key, session);

            let expected = input.1;
            let lines = lines
                .iter()
                .map(|line| {
                    line.iter
                        .filter_map(|e| match e {
                            IterEntry::Str(s) => Some(s),
                            IterEntry::Style(_) => None,
                        })
                        .collect::<String>()
                })
                .collect::<Vec<String>>()
                .join("\n");

            assert_eq!(expected, lines);
        }
    }

    #[test]
    fn word_split() {
        let size = layout(&["abc de"], (4, 10), Wrap::Normal);
        assert_eq!(size, (4, 2).into());
    }

    #[test]
    fn single_slice_single_line() {
        let size = layout(&["abc"], (10, 10), Wrap::Normal);
        assert_eq!(size, (3, 1).into());
    }

    #[test]
    fn multi_slice_single_line() {
        let inputs = ["abc", "de"];
        let size = layout(&inputs, (10, 10), Wrap::Normal);
        assert_eq!(size, (5, 1).into());
    }

    #[test]
    fn single_slice_multi_lines() {
        let size = layout(&["abc"], (1, 10), Wrap::Normal);
        assert_eq!(size, (1, 3).into());
    }

    #[test]
    fn multi_slice_multi_lines() {
        let inputs = ["abc", "de"];
        let size = layout(&inputs, (4, 10), Wrap::Normal);
        assert_eq!(size, (4, 2).into());
    }

    #[test]
    fn constraint_test() {
        let inputs = ["abcd"];
        let size = layout(&inputs, (1, 3), Wrap::Normal);
        assert_eq!(size, (1, 3).into());
    }
}
