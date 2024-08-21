use std::error::Error as StdError;
use std::fmt::{self, Display};

use anathema_templates::error::Error as TemplateError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Template(TemplateError),
    Notify(notify::Error),
    Widget(anathema_widgets::error::Error),
    Stop,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Template(template) => write!(f, "{template}"),
            Error::Stop => write!(f, "stopping"),
            Error::Notify(err) => write!(f, "{err}"),
            Error::Widget(err) => write!(f, "{err}"),
        }
    }
}

impl StdError for Error {}

impl From<TemplateError> for Error {
    fn from(err: TemplateError) -> Self {
        Self::Template(err)
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Self::Notify(err)
    }
}

impl From<anathema_widgets::error::Error> for Error {
    fn from(value: anathema_widgets::error::Error) -> Self {
        Self::Widget(value)
    }
}
