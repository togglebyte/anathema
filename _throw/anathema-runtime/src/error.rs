use std::error::Error as StdError;
use std::fmt::{self, Display};

use anathema_templates::error::Error as TemplateError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Template(TemplateError),
    Widget(anathema_widgets::error::Error),
    Notify(notify::Error),
    Stop,
    Reload,
    InvalidComponentName,
    Resolver(anathema_value_resolver::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Template(template) => write!(f, "{template}"),
            Self::Stop => write!(f, "stopping"),
            Self::Reload => write!(f, "reloading"),
            Self::Notify(err) => write!(f, "{err}"),
            Self::Widget(err) => write!(f, "{err}"),
            Self::Resolver(err) => write!(f, "{err}"),
            Self::InvalidComponentName => write!(f, "no such component"),
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

impl From<anathema_value_resolver::Error> for Error {
    fn from(value: anathema_value_resolver::Error) -> Self {
        Self::Resolver(value)
    }
}
