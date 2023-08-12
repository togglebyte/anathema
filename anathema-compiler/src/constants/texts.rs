use anathema_values::Slab;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextId(usize);

#[derive(Debug)]
pub struct Texts(Slab<String>);

impl Texts {
    pub(crate) fn empty() -> Self {
        Self(Slab::empty())
    }

    pub(crate) fn push(&mut self, text: String) -> TextId {
        TextId(self.0.push(text))
    }

    pub(crate) fn get(&self, id: TextId) -> Option<&String> {
        self.0.get(id.0)
    }
}


