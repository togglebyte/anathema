use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use crate::token::Operator;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub col: usize,
    pub src: String,
}

impl StdError for Error {}

impl Error {
    pub(crate) fn unexpected_eof(start: usize, src: &str) -> Self {
        let (line, col) = src_line_no(start, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::UnexpectedEof,
        }
    }

    pub(crate) fn unterminated_string(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.start, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::UnterminatedString,
        }
    }

    pub(crate) fn invalid_number(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::InvalidNumber,
        }
    }

    pub(crate) fn invalid_index(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::InvalidIndex,
        }
    }

    pub(crate) fn invalid_hex_value(range: Range<usize>, src: &str) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind: ErrorKind::InvalidHexValue,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let start_line = self.line;
        let lines = self
            .src
            .lines()
            .enumerate()
            .skip(start_line.saturating_sub(2))
            .take(3);

        let msg = match &self.kind {
            ErrorKind::UnterminatedString => "unterminated string".into(),
            ErrorKind::UnterminatedAttributes => "unterminated attributes (missing `]`)".into(),
            ErrorKind::UnterminatedElement => "unterminated element".into(),
            ErrorKind::InvalidToken { expected } => {
                format!("invalid token. expected: {expected}")
            }
            ErrorKind::InvalidNumber => "invalid number".into(),
            ErrorKind::InvalidIndex => "invalid index".into(),
            ErrorKind::InvalidPath => "invalid path".into(),
            ErrorKind::InvalidHexValue => "invalid hex value".into(),
            ErrorKind::UnexpectedEof => "unexpected end of file".into(),
            ErrorKind::TrailingPipe => "trailing pipe character".into(),
            ErrorKind::InvalidUnindent => {
                "dedent does not match previous indentation levels".into()
            }
            ErrorKind::InvalidOperator(op) => "invalid operator: {op}".into(),
            ErrorKind::UnexpectedToken(msg) => "unexpected token: {msg}".into(),
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

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnterminatedString,
    UnterminatedElement,
    UnterminatedAttributes,
    InvalidToken { expected: &'static str },
    InvalidNumber,
    InvalidIndex,
    InvalidHexValue,
    UnexpectedEof,
    TrailingPipe,
    InvalidUnindent,
    InvalidPath,
    InvalidOperator(Operator),
    UnexpectedToken(String),
}
