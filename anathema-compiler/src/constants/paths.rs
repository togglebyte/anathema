use anathema_values::{Path, PathId};

use super::Storage;

#[derive(Debug)]
pub struct Paths(Storage<Path>);

impl Paths {
    pub(crate) fn empty() -> Self {
        Self(Storage::empty())
    }

    pub(crate) fn push(&mut self, string: Path) -> PathId {
        PathId(self.0.push(string))
    }

    pub(crate) fn get(&self, index: PathId) -> Option<&Path> {
        self.0.get(index.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Path> + '_ {
        self.0.0.iter()
    }
}
