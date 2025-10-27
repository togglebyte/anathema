//! Compiling template source to blueprints and expressions.
pub use blueprints::Blueprint;
pub(crate) use blueprints::{Component, ControlFlow, For, Single, With};

pub use self::components::{ComponentBlueprintId, SourceKind};
pub use self::document::Document;
pub use self::expressions::{Expression, ExpressionId, Expressions};
pub use self::lexer::Lexer;
pub use self::primitives::Primitive;
pub use self::variables::Variables;

mod blueprints;
mod components;
mod document;
mod error;
pub(crate) mod expressions;
mod lexer;
mod primitives;
mod statements;
mod strings;
mod token;
mod variables;
