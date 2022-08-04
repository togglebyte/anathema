//! meh
pub(crate) mod lexer;
mod error;
mod parser;

pub use error::Error;
pub(crate) use parser::{Text, Parser};
