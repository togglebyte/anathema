use std::slice::IterMut;

use anathema_widget_core::WidgetContainer;

fn len_of_tree(widgets: &[WidgetContainer]) -> usize {
    let mut count = widgets.len();
    for widget in widgets {
        panic!("I removed the children!");
        // count += len_of_tree(&widget.children);
    }
    count
}

pub struct Frame {
    pub(crate) inner: Vec<WidgetContainer>,
}

impl Frame {
    pub fn empty() -> Self {
        Self { inner: vec![] }
    }

    pub fn push(&mut self, widget: WidgetContainer) {
        self.inner.push(widget);
    }

    pub fn count(&self) -> usize {
        len_of_tree(&self.inner)
    }
}

impl<'a> IntoIterator for &'a mut Frame {
    type IntoIter = IterMut<'a, WidgetContainer>;
    type Item = &'a mut WidgetContainer;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}
