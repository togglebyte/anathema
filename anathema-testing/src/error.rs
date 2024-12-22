use crate::parse::args::Arg;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) struct Error {
    pub(crate) kind: ErrorKind,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl std::error::Error for Error {}

impl Error {
    pub fn missing_section(line: usize) -> Self {
        Self {
            col: 1,
            line,
            kind: ErrorKind::MissingSection,
        }
    }
    
    pub fn parse_int(arg: Arg<'_>) -> Self {
        Self {
            col: arg.col,
            line: arg.line,
            kind: ErrorKind::ParseInt,
        }
    }

    pub fn missing_key(key: &str, line: usize) -> Self {
        Self {
            col: 1,
            line,
            kind: ErrorKind::MissingKey(key.into()),
        }
    }

    pub fn invalid_num_args(line: usize, expected: usize) -> Self {
        Self {
            col: 0,
            line,
            kind: ErrorKind::InvalidNumberOfArgs(expected)
        }
    }

    pub(crate) fn invalid_step(line: usize, ident: &str) -> Self {
        Self {
            col: 0,
            line,
            kind: ErrorKind::InvalidStep(ident.into()),
        }
    }

    pub(crate) fn invalid_keycode(arg: Arg<'_>) -> Self {
        Self {
            col: arg.col,
            line: arg.line,
            kind: ErrorKind::InvalidKeycode(arg.arg.into()),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error on line {}, and column {}: ", self.line, self.col)?;
        match &self.kind {
            ErrorKind::MissingSection => write!(f, "missing section"),
            ErrorKind::EmptyTestCase => write!(f, "empty test case"),
            ErrorKind::ParseInt => write!(f, "invalid number"),
            ErrorKind::InvalidNumberOfArgs(expected) => write!(f, "invalid number of arguments, expected {expected}"),
            ErrorKind::MissingKey(key) => write!(f, "missing value: {key}"),
            ErrorKind::InvalidStep(step) => write!(f, "{step} is not a valid action"),
            ErrorKind::InvalidKeycode(code) => write!(f, "{code} is not a valid keypress"),
        } 
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    MissingSection,
    EmptyTestCase,
    ParseInt,
    InvalidNumberOfArgs(usize),
    MissingKey(String),
    InvalidStep(String),
    InvalidKeycode(String),
}
