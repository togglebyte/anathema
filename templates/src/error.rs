//! An error
use std::fmt::{self, Display};

use widgets::Path;

/// A type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error.
#[derive(Debug)]
pub enum Error {
    /// Template node parse error.
    Parse(crate::parser::Error),
    /// Widget is not registered in lookup.
    UnregisteredWidget(String),
    /// Failed to create a widget through lookup.
    WidgetConstructionFailed(String),
    /// Binding has to be a valid string.
    BindingInvalidString,
    /// Data is not a collection, relevant to `for`-loops.
    NonCollectionValue,
    /// Include path is missing.
    MissingIncludePath,
    /// Missing condition for if-statment.
    MissingCondition,
    /// Missing identifier for a node that requires one.
    /// The only nodes that are excempt from having an id is:
    /// * for
    /// * if
    /// * else
    /// * elif
    /// * span
    MissingId,
    /// Value is required.
    ValueRequried,
    /// Target is already a transition.
    TargetIsTransition,
    /// Invalid text widget (not a text widget).
    InvalidTextWidget,
    /// Failed to lookup id
    IdNotFound(Path),
    /// Io Error.
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::UnregisteredWidget(e) => write!(f, "unregistered widget: {e}"),
            Self::WidgetConstructionFailed(e) => write!(f, "widget construction failed for {e}"),
            Self::BindingInvalidString => write!(f, "binding has to be a valid string"),
            Self::NonCollectionValue => write!(f, "the value is not a collection"),
            Self::MissingCondition => write!(f, "missing condition for if-statement"),
            Self::MissingId => write!(f, "the node is missing an identifier"),
            Self::MissingIncludePath => write!(f, "include path is missing"),
            Self::ValueRequried => write!(f, "value is required"),
            Self::TargetIsTransition => write!(f, "the selected value is already a transition"),
            Self::InvalidTextWidget => write!(f, "invalid text widget"),
            Self::IdNotFound(path) => write!(f, "node id was not found in the context: {path}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<crate::parser::Error> for Error {
    fn from(e: crate::parser::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

