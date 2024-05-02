use std::error::Error as StdError;
use std::fmt::{self, Display};

use anathema_templates::error::Error as TemplateError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Template(TemplateError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Template(template) => write!(f, "{template}"),
        }
    }
}

impl StdError for Error {}

impl From<TemplateError> for Error {
    fn from(err: TemplateError) -> Self {
        Self::Template(err)
    }
}
