use unicode_width::UnicodeWidthStr;

/// Split a string on either the whitespace character closest
/// to max length, or split it on max length if there is no
/// whitespace available.
fn split_to_len(line: &str, max_width: usize) -> (&str, &str) {
    let index = {
        let mut i = 0;
        while !line.is_char_boundary(max_width - i) {
            i += 1;
        }

        max_width - i
    };

    let split_pos = &line[..index].rfind(char::is_whitespace).map(|p| p + 1).unwrap_or(index);

    let (lhs, rhs) = line.split_at(*split_pos);
    (lhs, rhs)
}

/// Split lines to fit the screen.
fn split_line<'line, 'offset>(
    mut line: &'line str,
    max_width: usize,
    mut starting_offset: usize,
    keep_whitespace: bool,
) -> impl Iterator<Item = &'line str> {
    std::iter::from_fn(move || {
        if line.is_empty() {
            return None;
        }

        if line.len() + starting_offset > max_width && line.width() + starting_offset > max_width {

            // If the starting offset prevents the full word
            // from fitting, rather than breaking the word,
            // return empty string and try to split the line again
            // with the starting offest set to zero
            if starting_offset > 0 {
                if let Some(whitespace_pos) = line.find(|c: char| c.is_whitespace()) {
                    let total = starting_offset + whitespace_pos;
                    if total > max_width {
                        starting_offset = 0;
                        return Some("");
                    }
                }
            }


            log::info!("max: {} | start: {}", max_width, starting_offset);
            log::info!("line: {:?}", line);
            let (lhs, rhs) = split_to_len(line, max_width - starting_offset);
            log::info!("lhs: {:?}", lhs);
            log::info!("rhs: {:?}", rhs);
            starting_offset = 0;

            let lhs = match keep_whitespace {
                false => lhs.trim_start(),
                true => lhs,
            };

            line = match keep_whitespace {
                true => rhs.trim_start(),
                false => rhs.trim_start(),
            };

            return Some(lhs);
        } else {
            let ret_val = line;
            line = "";
            match keep_whitespace {
                true => Some(ret_val),
                false => Some(ret_val.trim_start()),
            }
        }
    })
}

pub fn split<'src>(
    src: &'src str,
    max_width: usize,
    starting_offset: usize,
    keep_whitespace: bool,
) -> impl Iterator<Item = &'src str> {
    src.split_inclusive('\n').map(move |l| split_line(l, max_width, starting_offset, keep_whitespace)).flatten()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_preserving_newlines() {
        let input = "hello\nworld\nlonger line here\nthe end";
        let res = split(input, 5, 0, false).collect::<Vec<_>>();

        let expected = "hello\n";
        let actual = res[0];

        assert_eq!(expected, actual);
    }

    #[test]
    fn split_preserving_whitespace() {
        let line = "     hello world";
        let max_width = 13;
        let s_split = split(line, max_width, 0, true);

        let result = s_split.collect::<Vec<_>>();
        assert_eq!(result[0], "     hello ");
        assert_eq!(result[1], "world");


        let line = "   x   ";
        let max_width = 5;
        let s_split = split(line, max_width, 0, true);

        let result = s_split.collect::<Vec<_>>();
        assert_eq!(result[0], "   x ");
    }

    #[test]
    fn test_split_nospace() {
        let line = "hello";
        let max_width = 4;
        let result = split(line, max_width, 0, false).collect::<Vec<_>>();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_split_prefix_space() {
        // Without an offset the first line "hello" fits
        // a width of five...
        let line = "hello";
        let max_width = 5;
        let result = split(line, max_width, 0, false).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);

        // ... however with an offset "hello" now spans
        // two lines
        let line = "hello";
        let max_width = 5;
        let result = split(line, max_width, 1, false).collect::<Vec<_>>();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn retain_whitespace() {
        let expected = "    hi";
        let mut actual = split(expected, expected.len(), 0, true).collect::<Vec<_>>();
        let actual = actual.pop().unwrap();

        assert_eq!(expected, actual);
    }
}
