use anathema_render::Style;
use anathema_values::{Attributes, Context, NodeId, ValueExpr, ValueRef};

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
pub struct RenameThis<T> {
    inner: Option<T>,
    expr: ValueExpr,
}

impl<T> RenameThis<T> {
    pub fn new(expr: ValueExpr) -> Self {
        Self { inner: None, expr }
    }
}

impl RenameThis<bool> {
    pub fn is_true(&self) -> bool {
        self.inner.unwrap_or(false)
    }
}

impl RenameThis<String> {
    pub fn string(&self) -> &String {
        static EMPTY: String = String::new();
        self.inner.as_ref().unwrap_or(&EMPTY)
    }
}

impl ValueResolver for RenameThis<String> {
    type Value = String;

    fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        self.inner = self.expr.eval_string(context, node_id);
    }
}

pub trait ValueResolver {
    type Value: for<'b> TryFrom<ValueRef<'b>>;

    fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>);
}

macro_rules! value_resolver_for_basetype {
    ($t:ty) => {
        impl ValueResolver for RenameThis<$t> {
            type Value = $t;

            fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
                let x = self.expr.to_string();
                let value_ref = match self.expr.eval_value_ref(context) {
                    Some(ValueRef::Deferred(path)) => context.state.get(&path, node_id),
                    val => val,
                };
                self.inner = value_ref.and_then(|v| Self::Value::try_from(v).ok());
            }
        }
    }
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
