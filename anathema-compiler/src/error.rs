use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

pub type Result<T> = std::result::Result<T, Error>;

pub(super) fn src_line_no(end: usize, src: &str) -> (usize, usize) {
    let mut line_no = 1;
    let mut pos = 0;

    while let Some(p) = &src[pos..end].find('\n') {
        pos += p + 1;
        line_no += 1;
    }

    // Set the column to at least one, as zero makes no
    // sense to the end user
    let col = 1 + end - pos;

    (line_no, col)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub col: usize,
    pub src: String,
}

impl StdError for Error {}

impl Error {
    pub(crate) fn unterminated_string(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.start, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::UnterminatedString }
    }

    pub(crate) fn invalid_number(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::InvalidNumber }
    }

    pub(crate) fn invalid_hex_value(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self { line, col, src: src.to_string(), kind: ErrorKind::InvalidHexValue }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let start_line = self.line;
        let lines = self.src.lines().enumerate().skip(start_line.saturating_sub(2)).take(3);

        let msg = match &self.kind {
            ErrorKind::UnterminatedString => "unterminated string".to_string(),
            ErrorKind::UnterminatedAttributes => "unterminated attributes (missing `]`)".to_string(),
            ErrorKind::UnterminatedElement => "unterminated element".to_string(),
            ErrorKind::InvalidToken { expected } => {
                format!("invalid token. expected: {expected}")
            }
            ErrorKind::InvalidNumber => "invalid number".to_string(),
            ErrorKind::InvalidHexValue => "invalid hex value".to_string(),
            ErrorKind::UnexpectedEnd => "unexpected end".to_string(),
            ErrorKind::TrailingPipe => "trailing pipe character".to_string(),
            ErrorKind::InvalidUnindent => "unindent does not match previous indentation levels".to_string(),
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
    UnterminatedAttributes,
    InvalidToken { expected: &'static str },
    InvalidNumber,
    InvalidHexValue,
    UnexpectedEnd,
    TrailingPipe,
    InvalidUnindent,
}
