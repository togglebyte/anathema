use anathema_state::{Color, Hex};
use anathema_value_resolver::{AttributeStorage, ValueKind};

pub use self::components::Components;
pub use self::elements::Elements;
use crate::{DirtyWidgets, WidgetTreeView};

mod components;
mod elements;

pub struct Children<'tree, 'bp>(Nodes<'tree, 'bp>);

impl<'tree, 'bp> Children<'tree, 'bp> {
    pub fn new(
        children: WidgetTreeView<'tree, 'bp>,
        attribute_storage: &'tree mut AttributeStorage<'bp>,
        dirty_widgets: &'tree mut DirtyWidgets,
    ) -> Self {
        Self(Nodes::new(children, attribute_storage, dirty_widgets))
    }

    pub fn elements(&mut self) -> Elements<'_, 'tree, 'bp> {
        Elements { elements: &mut self.0 }
    }

    pub fn components(&mut self) -> Components<'_, 'tree, 'bp> {
        Components { elements: &mut self.0 }
    }

    pub fn parent_path(&self) -> &[u16] {
        self.0.children.offset
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

macro_rules! impl_from_int {
    ($t:ty) => {
        impl<'a> From<$t> for QueryValue<'a> {
            fn from(value: $t) -> Self {
                Self::Int(value as i64)
            }
        }
    };
}

impl_from_int!(u8);
impl_from_int!(u16);
impl_from_int!(u32);
impl_from_int!(u64);
impl_from_int!(usize);
impl_from_int!(i8);
impl_from_int!(i16);
impl_from_int!(i32);
impl_from_int!(i64);
impl_from_int!(isize);

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
        self.a.filter(arg, attributes) && self.b.filter(arg, attributes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn filter_by_tag() {
        let tpl = "
        many
            many
                text '1'
            text '2'
            many
                many
                    text '3'
        ";

        crate::testing::with_template(tpl, |tree, attributes| {
            let mut dirty = DirtyWidgets::empty();
            let mut children = Children::new(tree, attributes, &mut dirty);
            let mut cntr = 0;
            children.elements().by_tag("text").each(|el, _| {
                assert_eq!(el.ident, "text");
                cntr += 1;
            });

            assert_eq!(cntr, 3);
        });
    }

    #[test]
    fn filter_by_tag_and_attribute() {
        let tpl = "
        many
            many
                text [a: 1] '1'
            text '2'
            many
                many
                    text [a: 1] '3'
        ";

        crate::testing::with_template(tpl, |tree, attributes| {
            let mut dirty = DirtyWidgets::empty();
            let mut children = Children::new(tree, attributes, &mut dirty);
            let mut cntr = 0;

            children.elements().by_tag("text").by_attribute("a", 1).each(|el, _| {
                assert_eq!(el.ident, "text");
                cntr += 1;
            });

            assert_eq!(cntr, 2);
        });
    }
}
