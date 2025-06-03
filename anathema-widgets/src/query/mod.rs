use anathema_state::{Color, Hex};
use anathema_value_resolver::{AttributeStorage, ValueKind};

pub use self::components::Components;
pub use self::elements::Elements;
use crate::WidgetTreeView;

mod components;
mod elements;

pub struct Children<'tree, 'bp>(Nodes<'tree, 'bp>);

impl<'tree, 'bp> Children<'tree, 'bp> {
    pub fn new(
        children: WidgetTreeView<'tree, 'bp>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        needs_layout: &'tree mut bool,
    ) -> Self {
        Self(Nodes::new(children, attribute_storage, needs_layout))
    }

    pub fn elements(&mut self) -> Elements<'_, 'tree, 'bp> {
        Elements { elements: &mut self.0 }
    }

    pub fn components(&mut self) -> Components<'_, 'tree, 'bp> {
        Components { elements: &mut self.0 }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum QueryValue<'a> {
    Str(&'a str),
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Hex(Hex),
    Color(Color),
}

impl<'a> From<&'a str> for QueryValue<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(value)
    }
}

impl<'a> From<bool> for QueryValue<'a> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl PartialEq<ValueKind<'_>> for QueryValue<'_> {
    fn eq(&self, other: &ValueKind<'_>) -> bool {
        match self {
            QueryValue::Str(lhs) => match other {
                ValueKind::Str(rhs) => lhs == rhs,
                _ => false,
            },
            QueryValue::Bool(lhs) => match other {
                ValueKind::Bool(rhs) => lhs == rhs,
                _ => false,
            },
            &QueryValue::Int(lhs) => match other {
                &ValueKind::Int(rhs) => lhs == rhs,
                _ => false,
            },
            &QueryValue::Float(lhs) => match other {
                &ValueKind::Float(rhs) => lhs == rhs,
                _ => false,
            },
            &QueryValue::Char(lhs) => match other {
                &ValueKind::Char(rhs) => lhs == rhs,
                _ => false,
            },
            &QueryValue::Hex(lhs) => match other {
                &ValueKind::Hex(rhs) => lhs == rhs,
                _ => false,
            },
            &QueryValue::Color(lhs) => match other {
                &ValueKind::Color(rhs) => lhs == rhs,
                _ => false,
            },
        }
    }
}

// -----------------------------------------------------------------------------
//   - Elements -
// -----------------------------------------------------------------------------
pub struct Nodes<'tree, 'bp> {
    children: WidgetTreeView<'tree, 'bp>,
    attributes: &'tree mut AttributeStorage<'bp>,
    needs_layout: &'tree mut bool,
}

impl<'tree, 'bp> Nodes<'tree, 'bp> {
    pub fn new(
        children: WidgetTreeView<'tree, 'bp>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        needs_layout: &'tree mut bool,
    ) -> Self {
        Self {
            children,
            attributes: attribute_storage,
            needs_layout,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Query -
// -----------------------------------------------------------------------------
pub struct Query<'el, 'tree, 'bp, F, T>
where
    F: Filter<'bp, Kind = T>,
{
    filter: F,
    elements: &'el mut Nodes<'tree, 'bp>,
}

impl<'el, 'tree, 'bp, F, T> Filter<'bp> for Query<'el, 'tree, 'bp, F, T>
where
    F: Filter<'bp, Kind = T>,
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
