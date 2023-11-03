use std::cell::RefCell;

pub use self::collection::Collection;
pub use self::id::NodeId;
pub use self::list::List;
pub use self::path::Path;
pub use self::scope::{Context, LocalScope};
pub use self::slab::Slab;
pub use self::state::{Change, State, StateValue};
pub use self::value::{Num, Owned, ValueRef};
pub use self::value_expr::{Deferred, Resolver, ValueExpr, ValueResolver};

pub mod hashmap;
mod path;

mod collection;
mod id;
mod list;
mod map;
mod scope;
mod slab;
mod state;
mod value;
mod value_expr;

pub type Attributes = hashmap::HashMap<String, ValueExpr>;

thread_local! {
    static DIRTY_NODES: RefCell<Vec<(NodeId, Change)>> = Default::default();
    static REMOVED_NODES: RefCell<Vec<NodeId>> = Default::default();
}

pub fn drain_dirty_nodes() -> Vec<(NodeId, Change)> {
    DIRTY_NODES.with(|nodes| nodes.borrow_mut().drain(..).collect())
}

pub fn remove_node(node: NodeId) {
    REMOVED_NODES.with(|nodes| nodes.borrow_mut().push(node));
}

#[cfg(any(feature = "testing", test))]
pub mod testing;

#[derive(Debug, Default)]
pub enum Value<T> {
    Dyn {
        inner: Option<T>,
        expr: ValueExpr,
    },
    Static(T),
    #[default]
    Empty,
}

impl<T> Value<T>
where
    T: DynValue,
{
    pub fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        T::resolve(self, context, node_id);
    }
}

impl<T> Value<T> {
    pub fn value_ref(&self) -> Option<&T> {
        match self {
            Self::Static(val) => Some(val),
            Self::Dyn { inner, .. } => inner.as_ref(),
            _ => None,
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl<T: Default + Copy> Value<T> {
    pub fn value(&self) -> Option<T> {
        match self {
            Self::Static(val) => Some(*val),
            &Self::Dyn { inner, .. } => inner,
            _ => None,
        }
    }

    pub fn value_or_default(&self) -> T {
        match self {
            Self::Static(val) => Some(*val),
            &Self::Dyn { inner, .. } => inner,
            _ => None,
        }
        .unwrap_or_else(T::default)
    }

    pub fn value_or(&self, default: T) -> T {
        match self {
            Self::Static(val) => Some(*val),
            &Self::Dyn { inner, .. } => inner,
            _ => None,
        }
        .unwrap_or(default)
    }

    pub fn value_or_else<F>(&self, default: F) -> T
    where
        F: Fn() -> T,
    {
        match self {
            Self::Static(val) => Some(*val),
            &Self::Dyn { inner, .. } => inner,
            _ => None,
        }
        .unwrap_or_else(default)
    }
}

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

impl DynValue for String {
    fn init_value(
        context: &Context<'_, '_>,
        node_id: Option<&NodeId>,
        expr: &ValueExpr,
    ) -> Value<Self> {
        let mut resolver = Resolver::new(context, node_id);
        let inner = resolver.resolve_string(expr);

        match resolver.is_deferred() {
            true => Value::Dyn {
                inner,
                expr: expr.clone(),
            },
            false => match inner {
                Some(val) => Value::Static(val),
                None => Value::Empty,
            },
        }
    }

    fn resolve(value: &mut Value<Self>, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        match value {
            Value::Dyn { inner, expr } => {
                *inner = Resolver::new(context, node_id).resolve_string(expr)
            }
            _ => {}
        }
    }
}

pub trait DynValue {
    fn init_value(
        context: &Context<'_, '_>,
        node_id: Option<&NodeId>,
        expr: &ValueExpr,
    ) -> Value<Self>
    where
        Self: Sized;

    fn resolve(value: &mut Value<Self>, context: &Context<'_, '_>, node_id: Option<&NodeId>)
    where
        Self: Sized;
}

#[macro_export]
macro_rules! impl_dyn_value {
    ($t:ty) => {
        impl DynValue for $t {
            fn init_value(
                context: &Context<'_, '_>,
                node_id: Option<&NodeId>,
                expr: &ValueExpr,
            ) -> Value<Self> {
                let mut resolver = Resolver::new(context, node_id);
                let inner = expr
                    .eval(&mut resolver)
                    .and_then(|v| Self::try_from(v).ok());

                match resolver.is_deferred() {
                    true => Value::Dyn {
                        inner,
                        expr: expr.clone(),
                    },
                    false => match inner {
                        Some(val) => Value::Static(val),
                        None => Value::Empty,
                    },
                }
            }

            fn resolve(
                value: &mut Value<Self>,
                context: &Context<'_, '_>,
                node_id: Option<&NodeId>,
            ) {
                match value {
                    Value::Dyn { inner, expr } => {
                        *inner = expr
                            .eval(&mut Resolver::new(context, node_id))
                            .and_then(|v| Self::try_from(v).ok())
                    }
                    _ => {}
                }
            }
        }
    };
}

impl_dyn_value!(bool);
impl_dyn_value!(anathema_render::Color);

impl_dyn_value!(usize);
impl_dyn_value!(u64);
impl_dyn_value!(u32);
impl_dyn_value!(u16);
impl_dyn_value!(u8);

impl_dyn_value!(isize);
impl_dyn_value!(i64);
impl_dyn_value!(i32);
impl_dyn_value!(i16);
impl_dyn_value!(i8);

impl_dyn_value!(f64);
impl_dyn_value!(f32);
