use crate::antstring::{AntString, Find};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static WORD_BOUNDARIES: &[char] = &[' ', '\n'];

/// Word wrapping.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Wrap {
    /// Output the text on a single line
    NoWrap,
    /// Fit as many characters as possible within the given constraint
    Break,
    /// Fit as many words as possible within the given constraint
    Word,
}

// Remove trailing control characters (i.e [`char::is_ascii_control`])
fn trim_trailing_control_chars<T>(input: &mut AntString<'_, T>) {
    if input.is_empty() {
        return;
    }
    let offset = input.len() - input.chars_rev().take_while(char::is_ascii_control).count();
    input.remove(offset..);
}

fn remove_newline_chars<T>(input: &mut AntString<'_, T>) {
    while let Some(pos) = input.find('\n') {
        input.remove(pos..=pos);
    }
}

// Trim excess white space (i.e any space followed by another space)
fn trim_excess_space<T>(input: &mut AntString<'_, T>) {
    let mut consume_whitespace = false;

    let mut from = 0;
    let mut count = 0;

    let mut remove_ranges = vec![];

    for (i, c) in input.char_indices() {
        if consume_whitespace {
            match c.is_whitespace() {
                true => count += c.len_utf8(),
                false => {
                    consume_whitespace = false;
                    if count > 0 {
                        remove_ranges.push(from..from + count);
                        count = 0;
                    }
                }
            }
        }

        if c.is_whitespace() && !consume_whitespace {
            consume_whitespace = true;
            from = i + c.len_utf8();
        }
    }

    if consume_whitespace {
        remove_ranges.push(from..from + count);
    }

    for range in remove_ranges.into_iter().rev() {
        input.remove(range);
    }
}

fn next_word_width<T>(input: AntString<'_, T>) -> usize {
    let index = input.find(WORD_BOUNDARIES).unwrap_or(input.len());
    input.get(..index).width()
}

// Find the end of the string and return that, which is the byte index
fn find_end_of_string<T>(input: &mut AntString<'_, T>, max_width: usize, word_wrap: bool) -> usize {
    let mut last_whitespace = None;
    let mut current_index = 0;
    let mut current_width = 0;
    let mut tracking_whitespace = false;

    for (i, c) in input.char_indices() {
        let char_len = c.len_utf8();
        let char_width = c.width().unwrap_or(0);

        if c.is_whitespace() {
            // track whitespace
            tracking_whitespace = true;
            last_whitespace = Some(i);
        } else if tracking_whitespace {
            tracking_whitespace = false;
            last_whitespace = Some(i);
        }

        if current_width + char_width > max_width {
            current_index = match (last_whitespace, word_wrap) {
                (Some(last), true) => current_index.min(last),
                (_, _) => current_index,
            };

            // Check if the next word is wider than max width.
            // If it is, then include the start of the word since it
            // can't fit by it self anyway.
            let next_width = next_word_width(input.get(current_index..));
            if next_width > max_width {
                current_index = i;
            }

            break;
        }

        current_width += char_width;
        current_index += char_len;
    }

    current_index
}

pub(crate) struct TextLayout {
    pub(crate) trim_start: bool,
    pub(crate) trim_end: bool,
    pub(crate) collapse_spaces: bool,
    ignore_newline: bool,
    wrap: Wrap,
    max_width: usize,
}

impl TextLayout {
    pub(crate) fn new(wrap: Wrap, max_width: usize) -> Self {
        Self { trim_start: true, trim_end: true, collapse_spaces: false, ignore_newline: false, wrap, max_width }
    }

    /// Return a list of strings where each string represents a new line
    ///
    /// Note: Since text has to be able to align to the centre or right it's not possible
    /// to have a dynamic size for the text, rather the text has to be sized
    /// by the constraints
    pub fn layout<'a, T>(&self, mut input: AntString<'a, T>) -> Vec<AntString<'a, T>> {
        if self.max_width == 0 {
            return vec![];
        }

        let mut ret = vec![];
        trim_trailing_control_chars(&mut input);

        if self.ignore_newline {
            remove_newline_chars(&mut input);
            return self._layout(input);
        }

        for s in input.lines() {
            let mut text = self._layout(s);
            ret.append(&mut text);
        }

        ret
    }

    fn _layout<'a, T>(&self, mut input: AntString<'a, T>) -> Vec<AntString<'a, T>> {
        let mut ret = vec![];

        if self.collapse_spaces {
            trim_excess_space(&mut input);
        }

        // Trim whitespace
        match (self.trim_start, self.trim_end) {
            (true, true) => input.trim(),
            (true, false) => input.trim_start(),
            (false, true) => input.trim_end(),
            (false, false) => {}
        }

        if input.width() < self.max_width {
            return vec![input];
        }

        // Make a new line for each iteration
        loop {
            // Trim whitespace
            match (self.trim_start, self.trim_end) {
                (true, true) => input.trim(),
                (true, false) => input.trim_start(),
                (false, true) => input.trim_end(),
                (false, false) => {}
            }

            // Find the end of the current string slice based on the
            // word wrapping method selected.
            let mut end_of_string = match self.wrap {
                Wrap::Word => find_end_of_string(&mut input, self.max_width, true),
                Wrap::Break => find_end_of_string(&mut input, self.max_width, false),
                Wrap::NoWrap => input.len().min(self.max_width),
            };

            if !self.ignore_newline {
                end_of_string = match input.get(..end_of_string).find('\n') {
                    Some(pos) => {
                        // Remove the newline char
                        input.remove(pos..=pos);
                        pos
                    }
                    None => end_of_string,
                }
            }

            let (mut left, right) = input.split_at(end_of_string);

            // If the end is at index zero there is no
            // way of splitting the string. This likely means the string doesn't fit 
            // the required size.
            //
            // In this case we return an empty string.
            if end_of_string == 0 {
                break;
            }

            input = right;
            trim_trailing_control_chars(&mut left);

            if self.trim_end {
                left.trim_end();
            }

            ret.push(left);

            // If there is no word wrapping: truncate
            // the string by returning it
            if let Wrap::NoWrap = self.wrap {
                return ret;
            }

            if input.is_empty() {
                break;
            }
        }

        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn breaking_word() {
        let input = AntString::new(["012345"]);
        let layout = TextLayout::new(Wrap::Break, 3);
        let actual = layout.layout(input);
        assert_eq!("012".to_string(), actual[0].to_string());
        assert_eq!("345".to_string(), actual[1].to_string());
    }

    #[test]
    fn newline() {
        let input = AntString::new(["ab\ncd"]);
        let max_width = 90;
        let layout = TextLayout::new(Wrap::Word, max_width);
        let actual = layout.layout(input);
        assert_eq!("ab".to_string(), actual[0].to_string());
        assert_eq!("cd".to_string(), actual[1].to_string());
    }

    #[test]
    fn multiple_newlines() {
        let s = "ab\n\n\ncd";
        let input = AntString::new([s]);
        let max_width = 90;
        let mut layout = TextLayout::new(Wrap::Word, max_width);
        layout.trim_start = false;
        layout.trim_end = false;
        let actual = layout.layout(input);
        assert_eq!("ab".to_string(), actual[0].to_string());
        assert_eq!("".to_string(), actual[1].to_string());
        assert_eq!("".to_string(), actual[2].to_string());
        assert_eq!("cd".to_string(), actual[3].to_string());
    }

    #[test]
    fn ignore_newline() {
        let input = AntString::new(["ab\ncd"]);
        let max_width = 90;
        let mut layout = TextLayout::new(Wrap::Word, max_width);
        layout.ignore_newline = true;
        let actual = layout.layout(input);
        assert_eq!("abcd".to_string(), actual[0].to_string());
    }

    #[test]
    fn remove_control_chars() {
        let mut s = String::new();
        s.push('a');
        s.push('\0');

        let mut input = AntString::new([s.as_str()]);
        assert_eq!(input.len(), 2);
        trim_trailing_control_chars(&mut input);
        assert_eq!(input.len(), 1);
    }

    #[test]
    fn remove_newlines() {
        let mut s = String::new();
        s.push('a');
        s.push('\n');
        s.push('a');
        s.push('\n');

        let mut input = AntString::new([s.as_str()]);
        assert_eq!(input.len(), 4);
        remove_newline_chars(&mut input);
        assert_eq!(input.len(), 2);
    }

    #[test]
    fn trim_excess_spaces() {
        let s = "  a  b  c        ";
        let mut input = AntString::new([s]);
        trim_excess_space(&mut input);
        assert_eq!(input.to_string(), " a b c ");
    }

    #[test]
    fn find_next_word_width() {
        let s = "aa bbbbb";
        let input = AntString::new([s]);
        let len = next_word_width(input);
        assert_eq!(len, 2);
    }

    #[test]
    fn trim_excessive_whitespace_to_fit() {
        // Given a string that is six cells wide with a max width
        // of five cells, and the aforementioned string ends with a whitespace,
        // then the whitespace should be trimmed
        let input = AntString::new(["hello  h", "ow  a", "re  you! "]);
        let max_width = 9;
        let mut layout = TextLayout::new(Wrap::Word, max_width);
        layout.trim_start = true;
        layout.trim_end = true;
        layout.collapse_spaces = true;
        let mut actual = layout.layout(input).into_iter();
        assert_eq!("hello how".to_string(), actual.next().unwrap().to_string());
        assert_eq!("are you!".to_string(), actual.next().unwrap().to_string());
    }

    #[test]
    fn break_long_words() {
        let input = AntString::new(["hello su", "uuuuuper", "looooooo", "ng ", "hi"]);

        let max_width = 8;
        let layout = TextLayout::new(Wrap::Word, max_width);
        let actual = layout.layout(input).into_iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n");
        let expected = "hello su\nuuuuuper\nlooooooo\nng hi".to_string();

        assert_eq!(actual, expected);

        let input = AntString::new(["hellosu", "uuuuuper", "looooooo", "ng ", "hi"]);

        let max_width = 8;
        let layout = TextLayout::new(Wrap::Word, max_width);
        let actual = layout.layout(input).into_iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n");
        let expected = "hellosuu\nuuuuperl\nooooooon\ng hi".to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn whitespace_aware_word_wrapping() {
        // * Split on space
        // * Preserve whitespace
        // * Max length = 5
        let input = AntString::new(["hello  world"]);

        let mut layout = TextLayout::new(Wrap::Word, 5);
        layout.trim_start = false;
        layout.trim_end = false;
        let output = layout.layout(input);
        let first = format!("{}", &output[0]);
        let second = format!("{}", &output[1]);
        let third = format!("{}", &output[2]);

        assert_eq!(first, "hello");
        assert_eq!(second, "  ");
        assert_eq!(third, "world");
    }

    #[test]
    fn whitespace_aware_word_wrapping_2() {
        // * Split on space
        // * Preserve whitespace
        // * Max length = 6
        let mut input = AntString::new([" world"]);
        let actual = find_end_of_string(&mut input, 6, true);
        let expected = 6;
        assert_eq!(actual, expected);
    }

    #[test]
    fn whitespace_aware_word_wrapping_3() {
        let mut input = AntString::new(["hello this is c ", "ool"]);
        let actual = find_end_of_string(&mut input, 15, true);
        let expected = 15;
        assert_eq!(actual, expected);
    }

    #[test]
    fn char_wrap_layout_ignoring_trailing_space() {
        let texts = [
            "helloworld",
            "helloworld ",
            "hello  world",
            "hello world  ",
            " hello world",
            " helloworld  ",
            " hello   world  ",
        ];

        for text in texts {
            let input = AntString::new([text]);
            let mut layout = TextLayout::new(Wrap::Word, 5);
            layout.trim_start = true;
            layout.trim_end = true;
            layout.collapse_spaces = true;
            let actual = layout.layout(input);
            let first = &actual[0].to_string();
            let second = &actual[1].to_string();

            assert_eq!(first, "hello");
            assert_eq!(second, "world");
        }
    }

    #[test]
    fn tomato_test() {
        let text = "🍅🍅🍅🍅🍅🍅🍅🍅🍅 🍅🍅🍅🍅🍅🍅🍅🍅🍅";
        let input = AntString::new([text]);
        let layout = TextLayout::new(Wrap::Word, 20);
        let actual = layout.layout(input);
        let first = &actual[0].to_string();
        let second = &actual[1].to_string();
        assert_eq!(first, "🍅🍅🍅🍅🍅🍅🍅🍅🍅");
        assert_eq!(second, "🍅🍅🍅🍅🍅🍅🍅🍅🍅");
    }

    #[test]
    fn truncate_on_nowrap() {
        let input = AntString::new(["t", "oo long and", " lots of strings"]);
        let layout = TextLayout::new(Wrap::NoWrap, 3);
        let mut output = layout.layout(input);

        assert_eq!(output.len(), 1);
        assert_eq!(output.remove(0).to_string(), "too");
    }

    #[test]
    fn zero_max_width() {
        let input = AntString::new(["hello world"]);
        let layout = TextLayout::new(Wrap::NoWrap, 0);
        let output = layout.layout(input);

        assert!(output.is_empty());
    }

    #[test]
    fn newline_word_wrapping() {
        let input = AntString::new(["This text.\nBreak"]);
        let layout = TextLayout::new(Wrap::Word, 8);
        let mut output = layout.layout(input);

        assert_eq!(output.remove(0).to_string(), "This");
        assert_eq!(output.remove(0).to_string(), "text.");
        assert_eq!(output.remove(0).to_string(), "Break");
    }

    #[test]
    fn dont_consume_all_newlines() {
        let text = r#"
A

B

C
        "#;
        let input = AntString::new([text]);
        let mut layout = TextLayout::new(Wrap::Word, 8);
        layout.trim_start = false;
        layout.trim_end = false;

        let output = layout.layout(input).into_iter().map(|s| s.to_string()).collect::<Vec<String>>().join("\n");
        assert_eq!(output, text);
    }

    #[test]
    fn split_on_invalid_char_boundary() {
        // Since even the first char in the input is larger
        // than the maximum width nothing can be returned.
        let input = AntString::new(["✨🍅✨"]);
        let mut layout = TextLayout::new(Wrap::Word, 1);
        let output = layout.layout(input);

        assert!(output.is_empty());
    }
}
