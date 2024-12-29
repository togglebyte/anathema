use std::marker::PhantomData;

use anathema_state::CommonVal;

pub use self::elements2::Elements;
pub use self::components::Components;
use crate::{AttributeStorage, DirtyWidgets, WidgetTreeView};

mod components;
mod elements;
mod elements2;

// -----------------------------------------------------------------------------
//   - Elements -
// -----------------------------------------------------------------------------
pub struct Nodes<'tree, 'bp> {
    children: WidgetTreeView<'tree, 'bp>,
    attributes: &'tree mut AttributeStorage<'bp>,
    dirty_widgets: &'tree mut DirtyWidgets,
}

impl<'tree, 'bp> Nodes<'tree, 'bp> {
    pub fn new(
        children: WidgetTreeView<'tree, 'bp>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        dirty_widgets: &'tree mut DirtyWidgets,
    ) -> Self {
        Self {
            children,
            attributes: attribute_storage,
            dirty_widgets,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Query -
// -----------------------------------------------------------------------------
pub struct Query<'el, 'tree, 'bp, F, T>
where
    F: Filter<'bp, Kind = T> + Copy,
{
    filter: F,
    elements: &'el mut Nodes<'tree, 'bp>,
}

impl<'el, 'tree, 'bp, F, T> Filter<'bp> for Query<'el, 'tree, 'bp, F, T>
where
    F: Filter<'bp, Kind = T> + Copy,
{
    type Kind = T;

    fn filter(&self, arg: &Self::Kind, attributes: &mut AttributeStorage<'bp>) -> bool {
        self.filter.filter(arg, attributes)
    }
}

// -----------------------------------------------------------------------------
//   - Filter -
// -----------------------------------------------------------------------------
pub trait Filter<'bp> {
    type Kind;

    fn filter(&self, arg: &Self::Kind, attributes: &mut AttributeStorage<'bp>) -> bool;
}

// -----------------------------------------------------------------------------
//   - Chain -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<'bp, A, B> Filter<'bp> for Chain<A, B>
where
    A: Filter<'bp>,
    B: Filter<'bp, Kind = A::Kind>,
{
    type Kind = A::Kind;

    fn filter(&self, arg: &Self::Kind, attributes: &mut AttributeStorage<'bp>) -> bool {
        self.a.filter(arg, attributes) | self.b.filter(arg, attributes)
    }
}
