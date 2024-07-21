use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

pub(crate) use self::parse::{src_line_no, ParseError, ParseErrorKind};

mod parse;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    CircularDependency,
    MissingComponent,
    EmptyTemplate,
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(err) => write!(f, "{err}"),
            Error::CircularDependency => write!(f, "circular dependency"),
            Error::MissingComponent => write!(f, "missing component"),
            Error::EmptyTemplate => write!(f, "empty template"),
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

impl StdError for Error {}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
