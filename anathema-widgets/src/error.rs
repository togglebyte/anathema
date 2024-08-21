use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidElement(String),
    TreeTransactionFailed,
    ComponentConsumed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidElement(el) => write!(f, "element `{el}` does not exist"),
            Error::TreeTransactionFailed => write!(
                f,
                "failed to insert into the widget tree (most likely the parent was removed)"
            ),
            Error::ComponentConsumed => write!(f, "this component has already been used"),
        }
    }
}

impl std::error::Error for Error {}
