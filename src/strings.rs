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

    let split_pos = &line[..index]
        .rfind(char::is_whitespace)
        .unwrap_or(index);

    let (lhs, rhs) = line.split_at(*split_pos);
    (lhs, rhs)
}

/// Split lines to fit the screen.
fn split_lines<'lines, 'offset>(mut line: &'lines str, max_width: usize, starting_offset: &'offset mut usize) -> Vec<&'lines str> {
    let mut lines = Vec::new();

    while line.width() + *starting_offset > max_width {
        let (lhs, rhs) = split_to_len(line, max_width - *starting_offset);
        *starting_offset = 0;

        let lhs = lhs.trim_start();
        if lhs.len() > 0 {
            lines.push(lhs);
        }

        line = rhs.trim_start();
    }

    lines.push(line.trim_start());

    lines
}

/// Split the input into lines that will fit on screen,
/// also break on newline chars.
pub fn split(input: &str, max_width: usize, mut starting_offset: usize) -> Vec<&str> {
    let lines = input
        .split_inclusive('\n')
        .map(|line| split_lines(line, max_width, &mut starting_offset))
        .flatten()
        .collect::<Vec<_>>();

    let lark = lines;

    let lines = lark;
    lines
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_nospace() {
        let line = "hello";
        let max_width = 4;
        let result = split(line, max_width, 0);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_split_prefix_space() {
        // Without an offset the first line "hello" fits
        // a width of five...
        let line = "hello";
        let max_width = 5;
        let result = split(line, max_width, 0);
        assert_eq!(result.len(), 1);

        // ... however with an offset "hello" now spans
        // two lines
        let line = "hello";
        let max_width = 5;
        let result = split(line, max_width, 1);
        assert_eq!(result.len(), 2);
    }
}

