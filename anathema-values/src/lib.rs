use std::cell::RefCell;

pub use self::id::NodeId;
pub use self::list::List;
pub use self::path::Path;
pub use self::scope::{Context, LocalScope, Value};
pub use self::slab::Slab;
pub use self::state::{Change, State, StateValue};
pub use self::value_expr::{ValueResolver, Resolver, Deferred, ValueExpr};
pub use self::value::{ValueRef, Num, Owned};
pub use self::collection::Collection;

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


// #[derive(Debug)]
// pub struct RenameThis<T> {
//     inner: Option<T>,
//     expr: ValueExpr
// }

// impl<T> RenameThis<T> {
//     pub fn new(expr: ValueExpr) -> Self {
//         Self {
//             inner: None,
//             expr,
//         }
//     }

//     pub fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) 
//     where
//         for<'b> T: TryFrom<ValueRef<'b>>,
//     {
//         let value_ref = match self.expr.eval_value_ref(context) {
//             Some(ValueRef::Deferred(path)) => context.state.get(&path, node_id),
//             val => val,
//         };
//         self.inner = value_ref.and_then(|v| T::try_from(v).ok());
//     }
// }

// impl RenameThis<bool> {
//     pub fn is_true(&self) -> bool {
//         self.inner.unwrap_or(false)
//     }
// }

// impl RenameThis<String> {
//     pub fn string(&self) -> &String {
//         static EMPTY: String = String::new();
//         self.inner.as_ref().unwrap_or(&EMPTY)
//     }
// }


// // TODO: this is a hack while trying to figure out what kind of value types to have
// #[derive(Debug)]
// pub struct TextVal {
//     inner: Option<String>,
//     expr: ValueExpr
// }

// impl TextVal {
//     pub fn new(expr: ValueExpr) -> Self {
//         Self {
//             inner: None,
//             expr,
//         }
//     }

//     pub fn resolve(&mut self, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
//         self.inner = self.expr.eval_string(context, node_id);
//     }

//     pub fn string(&self) -> &String {
//         static EMPTY: String = String::new();
//         self.inner.as_ref().unwrap_or(&EMPTY)
//     }
// }
