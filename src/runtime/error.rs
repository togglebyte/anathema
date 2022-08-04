use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Template error
    Template(crate::templates::error::Error),
    /// IO Error
    Io(std::io::Error),
    /// Serde error
    #[cfg(feature = "json")]
    Serde(serde_json::Error),
    /// No root widget container. This happens if the template was empty.
    MissingRoot,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Template(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            #[cfg(feature = "json")]
            Self::Serde(e) => write!(f, "{e}"),
            Self::MissingRoot => write!(f, "missing root widget"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<crate::templates::error::Error> for Error {
    fn from(e: crate::templates::error::Error) -> Self {
        Self::Template(e)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}
