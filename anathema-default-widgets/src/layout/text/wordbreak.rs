use anathema_geometry::Size;
use anathema_store::tree::ValueId;
use anathema_widgets::layout::{Entry, TextIndex, TextSession};
use unicode_width::UnicodeWidthStr;

use super::ProcessResult;

#[derive(Debug)]
pub struct WordBreakLayout<'a> {
    session: TextSession<'a>,
    index: TextIndex,
    max_size: Size,
    size: Size,
    line_width: usize,
    last_word_width: usize,
    pos: usize,
}

impl<'a> WordBreakLayout<'a> {
    pub fn new(max_size: impl Into<Size>, session: TextSession<'a>, index: TextIndex) -> Self {
        Self {
            session,
            index,
            max_size: max_size.into(),
            size: Size { width: 0, height: 1 },
            line_width: 0,
            last_word_width: 0,
            pos: 0,
        }
    }

    fn update_line_width(&mut self, used_width: usize) {
        self.size.width = self.size.width.max(used_width);
        let layout = self.session.layout.slice_mut(self.index.layout);
        match &mut layout[self.last_word_width] {
            Entry::LineWidth(width) => *width = used_width as u32,
            _ => unreachable!(),
        }
        self.last_word_width = layout.len() - 1;
    }

    pub fn process(&mut self, mut text: &str) -> ProcessResult {
        if text.is_empty() {
            return ProcessResult::Continue;
        }

        let origin = text;

        while text.width() > self.max_size.width - self.line_width {
            let Some((i, _)) = text.char_indices().last() else { return ProcessResult::Done };
            text = &text[..i];
        }

        self.session.bytes.extend(text.bytes());

        self.session.layout.push(Entry::Str(self.pos, self.pos + text.len()));
        self.pos += text.len();

        let width = text.width();

        // No more room so insert a newline
        if width == 0 {
            self.session.layout.push(Entry::Newline);
            self.session.layout.push(Entry::LineWidth(0));
            self.update_line_width(self.line_width);
            if self.size.height >= self.max_size.height {
                return ProcessResult::Done;
            }
            self.size.height += 1;
            self.line_width = 0;
        }

        self.line_width += width;

        self.process(&origin[text.len()..]);

        ProcessResult::Continue
    }

    pub fn finish(&mut self) {
        self.update_line_width(self.line_width);
    }

    pub fn size(mut self) -> Size {
        self.size.width = self.size.width.max(self.line_width);
        if self.size.width == 0 {
            self.size = Size::ZERO;
        }
        self.size
    }

    pub(crate) fn set_style(&mut self, style: ValueId) {
        self.session.layout.push(Entry::SetStyle(style));
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
        let inputs: &[(&[&str], &str)] = &[
            (&["12", "345　12", "345 "], "12345\n　123\n45 "),
            (&[" 🐇🐇🐇", "🐇🐇 12", "345 "], " 🐇🐇\n🐇🐇\n🐇 12\n345 "),
            (&["1", "23", "45 12", "345 "], "12345\n 1234\n5 "),
            (&["12345 abcde "], "12345\n abcd\ne "),
            (&["onereallylongword"], "onere\nallyl\nongwo\nrd"),
            (&["ahello do the"], "ahell\no do \nthe"),
            (&["hello do the"], "hello\n do t\nhe"),
        ];

        let mut text_buffer = TextBuffer::empty();

        for input in inputs {
            let size = (5, 10);

            let mut session = text_buffer.session();
            let key = session.new_key();
            let mut layout = TextLayout::new(size, Wrap::WordBreak, session, key);

            // Layout
            for part in input.0 {
                layout.process(part);
            }

            layout.finish();
            layout.size();

            // Read
            let session = text_buffer.session();
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
    fn single_slice_single_line() {
        let size = layout(&["abc"], (10, 10), Wrap::WordBreak);
        assert_eq!(size, (3, 1).into());
    }

    #[test]
    fn multi_slice_single_line() {
        let inputs = ["abc", "de"];
        let size = layout(&inputs, (10, 10), Wrap::WordBreak);
        assert_eq!(size, (5, 1).into());
    }

    #[test]
    fn single_slice_multi_lines() {
        let size = layout(&["abc"], (1, 10), Wrap::WordBreak);
        assert_eq!(size, (1, 3).into());
    }

    #[test]
    fn multi_slice_multi_lines() {
        let inputs = ["abc", "de"];
        let size = layout(&inputs, (4, 10), Wrap::WordBreak);
        assert_eq!(size, (4, 2).into());
    }

    #[test]
    fn constraint_test() {
        let inputs = ["abcd"];
        let size = layout(&inputs, (1, 3), Wrap::WordBreak);
        assert_eq!(size, (1, 3).into());
    }
}
