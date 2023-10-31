use anathema_render::Style;
use anathema_values::{Attributes, Context, NodeId, Resolver, ValueExpr, ValueRef};

pub mod contexts;
pub mod error;
mod factory;
pub mod generator;
pub mod layout;
mod values;
mod widget;

// #[cfg(feature = "testing")]
// pub mod testing;

pub use generator::Nodes;

pub use crate::factory::{Factory, FactoryContext, WidgetFactory};
pub use crate::layout::{Align, Axis, Direction, LocalPos, Padding, Pos, Region};
pub use crate::values::{Color, Display};
pub use crate::widget::{AnyWidget, Widget, WidgetContainer};

#[derive(Debug)]
pub enum Value<T> {
    Dyn { inner: Option<T>, expr: ValueExpr },
    Static(T),
    Empty,
}

impl<T> Value<T>
where
    T: for<'b> TryFrom<ValueRef<'b>>,
{
    pub fn new(expr: ValueExpr, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> Self {
        let mut resolver = Resolver::new(context, node_id);

        let inner = expr.eval(&mut resolver).and_then(|v| T::try_from(v).ok());

        match resolver.is_deferred() {
            true => Self::Dyn { inner, expr },
            false => match inner {
                Some(val) => Self::Static(val),
                None => Self::Empty,
            },
        }
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Static(val) => Some(val),
            Self::Dyn { inner, .. } => inner.as_ref(),
            _ => None,
        }
    }
}

// impl From<T> for Value<T> {
//     fn from(val: T) -> Self {
//         Self::Static(val)
//     }
// }

impl Value<bool> {
    pub fn is_true(&self) -> bool {
        match self {
            Self::Dyn { inner, .. } => inner.unwrap_or(false),
            Self::Static(b) => *b,
            Self::Empty => false,
        }
    }
}

impl Value<String> {
    pub fn string(&self) -> &str {
        static EMPTY: &str = "";
        match self {
            Self::Static(s) => s,
            Self::Dyn { inner: Some(s), .. } => s,
            Self::Dyn { inner: None, .. } => EMPTY,
            Self::Empty => EMPTY,
        }
    }
}

impl ValueResolver for Value<String> {
    type Value = String;

    fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        match self {
            Self::Dyn { inner, expr } => {
                *inner = Resolver::new(context, node_id).resolve_string(expr)
            }
            _ => {}
        }
    }
}

pub trait ValueResolver {
    type Value: for<'b> TryFrom<ValueRef<'b>>;

    fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>);
}

macro_rules! value_resolver_for_basetype {
    ($t:ty) => {
        impl ValueResolver for Value<$t> {
            type Value = $t;

            fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
                match self {
                    Self::Dyn { inner, expr } => {
                        *inner = expr
                            .eval(&mut Resolver::new(context, node_id))
                            .and_then(|v| Self::Value::try_from(v).ok())
                    }
                    _ => {}
                }
            }
        }
    };
}

value_resolver_for_basetype!(bool);
value_resolver_for_basetype!(Color);

value_resolver_for_basetype!(usize);
value_resolver_for_basetype!(u64);
value_resolver_for_basetype!(u32);
value_resolver_for_basetype!(u16);
value_resolver_for_basetype!(u8);

value_resolver_for_basetype!(isize);
value_resolver_for_basetype!(i64);
value_resolver_for_basetype!(i32);
value_resolver_for_basetype!(i16);
value_resolver_for_basetype!(i8);

value_resolver_for_basetype!(f64);
value_resolver_for_basetype!(f32);
