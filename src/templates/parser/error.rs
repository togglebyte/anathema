use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use super::lexer::TokenKind;

pub type Result<T> = std::result::Result<T, Error>;

fn src_line_no(end: usize, src: &str) -> (usize, usize) {
    let mut line_no = 1;
    let mut pos = 0;

    while let Some(p) = &src[pos..end].find('\n') {
        pos += p + 1;
        line_no += 1;
    }

    let col = end - pos;

    (line_no, col)
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub col: usize,
    pub src: String,
}

impl StdError for Error {}

impl Error {
    pub(crate) fn invalid_token(
        range: Range<usize>,
        src: &str,
        unexpected: TokenKind<'_>,
        expected: &'static str,
    ) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::InvalidToken { unexpected: unexpected.to_string(), expected },
        }
    }

    pub(crate) fn unterminated_string(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::UnterminatedString }
    }

    pub(crate) fn unterminated_element(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::UnterminatedElement }
    }

    pub(crate) fn invalid_number(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::InvalidNumber }
    }

    pub(crate) fn invalid_hex_value(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::InvalidHexValue }
    }

    pub(crate) fn invalid_attribute(range: Range<usize>, src: &str, name: &str, value: Option<&str>) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::InvalidAttribute { name: name.to_string(), value: value.map(|s| s.to_string()) },
        }
    }

    pub(crate) fn unexpected_end(src: &str) -> Self {
        let (line, col) = src_line_no(src.len(), src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::UnexpectedEnd }
    }

    pub(crate) fn trailing_pipe(pos: usize, src: &str) -> Self {
        let (line, col) = src_line_no(pos, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::TrailingPipe }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let start_line = self.line;
        let lines = self.src.lines().enumerate().skip(start_line.saturating_sub(2)).take(3);

        let msg = match &self.kind {
            ErrorKind::UnterminatedString => "unterminated string".to_string(),
            ErrorKind::UnterminatedElement => "unterminated element".to_string(),
            ErrorKind::InvalidToken { unexpected, expected } => {
                format!("invalid token. got: {unexpected:?}, expected: {expected}")
            }
            ErrorKind::InvalidNumber => "invalid number".to_string(),
            ErrorKind::InvalidAttribute { name, value: Some(value) } => format!("invalid attribute: {name}: {value}"),
            ErrorKind::InvalidAttribute { name, value: None } => format!("invalid attribute: {name}"),
            ErrorKind::InvalidHexValue => "invalid hex value".to_string(),
            ErrorKind::UnexpectedEnd => "unexpected end".to_string(),
            ErrorKind::TrailingPipe => "trailing pipe character".to_string(),
        };

        writeln!(f, "error on line {start_line}: {msg}")?;

        for (no, line) in lines {
            let no = no + 1;
            let mark = if self.line == no { "-> " } else { "   " };
            writeln!(f, "{mark}{no} {line}")?;
            if self.line == no {
                writeln!(f, "{:_<width$}|", "_", width = self.col)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UnterminatedString,
    UnterminatedElement,
    InvalidToken { unexpected: String, expected: &'static str },
    InvalidNumber,
    InvalidHexValue,
    InvalidAttribute { name: String, value: Option<String> },
    UnexpectedEnd,
    TrailingPipe,
}
