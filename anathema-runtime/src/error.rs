pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!()
    }
}

impl std::error::Error for Error {}
