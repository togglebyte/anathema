use super::AntString;

pub trait Sealed {}

impl<'a, T> Sealed for AntString<'a, T> {}
