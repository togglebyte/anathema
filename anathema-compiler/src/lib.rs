//! Compiling template source to blueprints and expressions.
pub use self::components::{ComponentBlueprintId, SourceKind};
pub use self::document::Document;
pub use self::variables::Variables;
pub use crate::colors::{Color, FromColor};
pub use crate::hex::Hex;

pub mod blueprints;
mod colors;
mod components;
mod document;
mod error;
pub mod expressions;
mod hex;
mod lexer;
mod statements;
mod strings;
mod token;
mod variables;
