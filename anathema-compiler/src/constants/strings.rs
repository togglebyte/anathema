use super::Storage;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StringId(usize);

#[derive(Debug)]
pub struct Strings(Storage<String>);

impl Strings {
    pub(crate) fn empty() -> Self {
        Self(Storage::empty())
    }

    pub(crate) fn push(&mut self, string: String) -> StringId {
        StringId(self.0.push(string))
    }

    pub(crate) fn get(&self, index: StringId) -> Option<&String> {
        self.0.get(index.0)
    }
}
