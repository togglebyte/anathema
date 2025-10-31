use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

pub(crate) use self::parse::{ParseError, ParseErrorKind, src_line_no};

mod parse;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    pub template_path: Option<PathBuf>,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(path: Option<PathBuf>, kind: impl Into<ErrorKind>) -> Self {
        Self {
            kind: kind.into(),
            template_path: path,
        }
    }

    pub fn no_template(kind: ErrorKind) -> Self {
        Self {
            kind,
            template_path: None,
        }
    }

    pub fn path(&self) -> String {
        match self.template_path.as_ref() {
            Some(path) => format!("{}", path.display()),
            None => String::new(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl StdError for Error {}

#[derive(Debug)]
pub enum ErrorKind {
    ParseError(ParseError),
    CircularDependency,
    MissingComponent(String),
    EmptyTemplate,
    EmptyBody,
    InvalidStatement(String),
    Io(std::io::Error),
    GlobalAlreadyAssigned(String),
}

impl ErrorKind {
    pub fn to_error(self, template: Option<PathBuf>) -> Error {
        Error::new(template, self)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::ParseError(err) => write!(f, "{err}"),
            ErrorKind::CircularDependency => write!(f, "circular dependency"),
            ErrorKind::MissingComponent(name) => write!(f, "`@{name}` is not a registered component"),
            ErrorKind::EmptyTemplate => write!(f, "empty template"),
            ErrorKind::EmptyBody => write!(f, "if or else node has no children"),
            ErrorKind::InvalidStatement(stmt) => write!(f, "invalid statement: {stmt}"),
            ErrorKind::Io(err) => write!(f, "{err}"),
            ErrorKind::GlobalAlreadyAssigned(name) => write!(f, "global value `{name}` already assigned"),
        }
    }
}

impl From<ParseError> for ErrorKind {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
