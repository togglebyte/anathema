use std::fmt::Display;
use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub path: Option<PathBuf>,
}

impl Error {
    pub(crate) fn transaction_failed() -> Self {
        Self {
            kind: ErrorKind::TreeTransactionFailed,
            path: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    InvalidElement(String),
    TreeTransactionFailed,
    ComponentConsumed(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidElement(el) => write!(f, "element `{el}` does not exist"),
            ErrorKind::TreeTransactionFailed => write!(
                f,
                "failed to insert into the widget tree (most likely the parent was removed)"
            ),
            ErrorKind::ComponentConsumed(name) => write!(f, "`@{name}` has already been used"),
        }
    }
}

impl std::error::Error for Error {}
