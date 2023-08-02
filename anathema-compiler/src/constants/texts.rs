use anathema_values::Slab;
use anathema_widget_core::TextPath;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextId(usize);

#[derive(Debug)]
pub struct Texts(Slab<TextPath>);

impl Texts {
    pub(crate) fn empty() -> Self {
        Self(Slab::empty())
    }

    pub(crate) fn push(&mut self, text: TextPath) -> TextId {
        TextId(self.0.push(text))
    }

    pub(crate) fn get(&self, id: TextId) -> Option<&TextPath> {
        self.0.get(id.0)
    }
}


