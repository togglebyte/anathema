use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use crate::token::Operator;

// Line number and column starts at one, not zero,
// because actual humans might read this
pub(crate) fn src_line_no(end: usize, src: &str) -> (usize, usize) {
    let mut line_no = 1;
    let mut pos = 0;

    while let Some(p) = &src[pos..end].find('\n') {
        pos += p + 1;
        line_no += 1;
    }

    // Set the column to at least one, as zero makes no sense to the end user
    let col = 1 + end - pos;

    (line_no, col)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub line: usize,
    pub col: usize,
    pub src: String,
}

impl StdError for ParseError {}

impl ParseError {
    pub(crate) fn new(range: Range<usize>, src: &str, kind: ParseErrorKind) -> Self {
        let (line, col) = src_line_no(range.end, src);
        Self {
            line,
            col,
            src: src.to_string(),
            kind,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let start_line = self.line;
        let lines = self.src.lines().enumerate().skip(start_line.saturating_sub(2)).take(3);

        let msg = match &self.kind {
            ParseErrorKind::UnterminatedString => "unterminated string".into(),
            ParseErrorKind::UnterminatedAttributes => "unterminated attributes (missing `]`)".into(),
            ParseErrorKind::UnterminatedAssociation => "unterminated association (missing `)`)".into(),
            ParseErrorKind::UnterminatedElement => "unterminated element".into(),
            ParseErrorKind::InvalidToken { expected } => {
                format!("invalid token (expected: \"{expected}\")")
            }
            ParseErrorKind::InvalidNumber => "invalid number".into(),
            ParseErrorKind::InvalidIndex => "invalid index".into(),
            ParseErrorKind::InvalidPath => "invalid path".into(),
            ParseErrorKind::InvalidHexValue => "invalid hex value".into(),
            ParseErrorKind::UnexpectedEof => "unexpected end of file".into(),
            ParseErrorKind::TrailingPipe => "trailing pipe character".into(),
            ParseErrorKind::InvalidDedent => "dedent does not match previous indentation levels".into(),
            ParseErrorKind::InvalidOperator(_op) => "invalid operator: {op}".into(),
            ParseErrorKind::UnexpectedToken(_msg) => "unexpected token: {msg}".into(),
            ParseErrorKind::InvalidKey => todo!(),
        };

        writeln!(f, "error on line {start_line}: {msg}")?;

        for (no, line) in lines {
            let no = no + 1;
            let mark = if self.line == no { "-> " } else { "   " };
            let mark_line = format!("{mark}{no}");
            writeln!(f, "{mark_line} {line}")?;

            // TODO:
            // This has a bug, it "points" to the wrong value.
            // This is most likely because the offsets are wrong.
            //
            // if self.line == no {
            //     writeln!(f, "{:_<width$}|", "_", width = self.col + mark_line.len())?;
            // }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    UnterminatedString,
    UnterminatedElement,
    UnterminatedAttributes,
    UnterminatedAssociation,
    InvalidToken { expected: &'static str },
    InvalidNumber,
    InvalidIndex,
    InvalidHexValue,
    UnexpectedEof,
    TrailingPipe,
    InvalidDedent,
    InvalidPath,
    InvalidOperator(Operator),
    UnexpectedToken(String),
    InvalidKey,
}
