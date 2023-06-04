pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Compiler error
    #[error("compiler error: {0}")]
    CompilerError(#[from] anathema_compiler::error::Error),
}

