use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::ControlFlow;

use anathema_state::StateId;
use anathema_store::tree::visitor::NodeVisitor;

use crate::{WidgetContainer, WidgetId, WidgetKind, WidgetTreeView};

// TODO
// Test this with
// * One component
// * Many components
// * No components that accepts focus

pub struct Index {
    path: Box<[u16]>,
    index: u16,
    pub widget_id: WidgetId,
    pub state_id: StateId,
}

impl Index {
    fn to_ref(&self) -> IndexRef<'_> {
        IndexRef {
            path: &self.path,
            index: self.index,
        }
    }
}

impl Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {:?}", self.index, self.path)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct IndexRef<'a> {
    path: &'a [u16],
    index: u16,
}

impl IndexRef<'_> {
    fn to_owned(self, widget_id: WidgetId, state_id: StateId) -> Index {
        Index {
            index: self.index,
            path: self.path.into(),
            widget_id,
            state_id,
        }
    }
}

impl PartialOrd for IndexRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ord = match self.index.cmp(&other.index) {
            ord @ (Ordering::Less | Ordering::Greater) => ord,
            Ordering::Equal => self.path.cmp(&other.path),
        };
        Some(ord)
    }
}

#[derive(Debug)]
pub enum Direction {
    Forward,
    Backward,
}

pub struct TabIndex<'a, 'bp> {
    pub previous: Option<Index>,
    pub current: &'a mut Option<Index>,
    tree: WidgetTreeView<'a, 'bp>,
    pub changed: &'a mut bool,
}

impl<'a, 'bp> TabIndex<'a, 'bp> {
    pub fn new(current: &'a mut Option<Index>, tree: WidgetTreeView<'a, 'bp>, changed: &'a mut bool) -> Self {
        Self {
            current,
            previous: None,
            tree,
            changed,
        }
    }

    pub fn consume(mut self) -> Option<Index> {
        self.previous.take()
    }

    pub fn next(&mut self) {
        self.find_component(Direction::Forward);
    }

    pub fn prev(&mut self) {
        self.find_component(Direction::Backward);
    }

    fn find_component(&mut self, dir: Direction) {
        let values = self.tree.values.iter();

        let mut next_index = NextIndex {
            dir: Direction::Forward,
            origin: self.current.as_ref().map(|i| i.to_ref()),
            next: None,
        };

        let mut smallest_index = None;
        let mut largest_index = None;

        let now = std::time::Instant::now();

        for (path, container) in values {
            match &container.kind {
                crate::WidgetKind::Component(component) => {
                    if component.dyn_component.any_accept_focus() {
                        let index = IndexRef {
                            path,
                            index: component.tabindex,
                        };

                        // Keep track of the smallest index
                        match &mut smallest_index {
                            Some(smallest) if *smallest > index => *smallest = index,
                            None => smallest_index = Some(index),
                            Some(_) => {}
                        }

                        // Keep track of the largest index
                        match &mut largest_index {
                            Some(largest) if *largest < index => *largest = index,
                            None => largest_index = Some(index),
                            Some(_) => {}
                        }

                        // Skip the current index
                        match &mut next_index.origin {
                            Some(origin) => match dir {
                                Direction::Forward if *origin >= index => continue,
                                Direction::Backward if *origin <= index => continue,
                                _ => {}
                            },
                            Some(_) | None => (),
                        }

                        match &mut next_index.next {
                            Some(next) => match dir {
                                Direction::Forward if *next > index => *next = index,
                                Direction::Backward if *next < index => *next = index,
                                _ => {}
                            },
                            None => next_index.next = Some(index),
                        }
                    }
                }
                _ => {}
            }
        }

        // Handle wrapping around.
        // I.e if the direction is forward and the current tab index is the same
        // as the max index, then set the next index to be the smallest index
        if let Some(origin) = next_index.origin.take() {
            let largest_index = largest_index.expect("if there is a next, there is a largest");
            let smallest_index = smallest_index.expect("if there is a largest, there is a smallest");
            match dir {
                Direction::Forward if origin == largest_index => { next_index.next.replace(smallest_index); },
                Direction::Backward if origin == smallest_index => { next_index.next.replace(largest_index); },
                _ => {},
            }
        }

        let Some(mut next) = next_index.next.take() else { return };

        let Some((widget_id, value)) = self.tree.get_node_and_value(next.path) else { return };
        let WidgetKind::Component(comp) = &value.kind else { return };
        let next = IndexRef::to_owned(next, widget_id, comp.state_id);

        self.previous = self.current.replace(next);
        *self.changed = true;

        crate::awful_debug!("{:?}", now.elapsed());
    }
}

#[derive(Debug)]
struct NextIndex<'a> {
    dir: Direction,
    // The current index, before trying to find the next one
    origin: Option<IndexRef<'a>>,
    next: Option<IndexRef<'a>>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aaa() {
        let a = IndexRef {
            path: Some(&[0]),
            index: 1,
        };

        let b = IndexRef {
            path: Some(&[0, 0]),
            index: 1,
        };

        match a.partial_cmp(&b) {
            Some(Ordering::Less) => panic!("less"),
            Some(Ordering::Greater) => panic!("greater"),
            Some(Ordering::Equal) => panic!("equal"),
            None => todo!(),
        }
    }
}
