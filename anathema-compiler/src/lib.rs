pub mod error;

pub(crate) mod compiler;
pub(crate) mod lexer;
pub(crate) mod parsing;
mod constants;

pub use compiler::Instruction;

pub use crate::constants::Constants;
pub use constants::{ValueId, StringId, CondId};

/// Compile source into instructions and constants.
pub fn compile(src: &str) -> error::Result<(Vec<Instruction>, Constants)> {
    let lexer = lexer::Lexer::new(src);
    let mut constants = Constants::new();
    let parser = parsing::parser::Parser::new(lexer, &mut constants)?;
    let expressions = parser.collect::<error::Result<Vec<_>>>()?;
    let optimizer = compiler::Optimizer::new(expressions);
    let expressions = optimizer.optimize();
    let compiler = compiler::Compiler::new(expressions);
    Ok((compiler.compile()?, constants))
}
