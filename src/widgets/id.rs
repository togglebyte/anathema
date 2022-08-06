use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use super::value::Value;

// -----------------------------------------------------------------------------
//     - Node id -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum NodeId {
    Value(Value),
    Auto(u64),
}

impl NodeId {
    pub fn auto() -> Self {
        static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed);
        Self::Auto(id)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(val) => write!(f, "{}", val),
            Self::Auto(val) => write!(f, "Auto ({})", val),
        }
    }
}

impl PartialEq<NodeId> for &str {
    fn eq(&self, rhs: &NodeId) -> bool {
        match rhs {
            NodeId::Value(Value::String(s)) => s == self,
            _ => false,
        }
    }
}

impl PartialEq<NodeId> for str {
    fn eq(&self, rhs: &NodeId) -> bool {
        match rhs {
            NodeId::Value(Value::String(s)) => s == self,
            _ => false,
        }
    }
}

impl<T> From<T> for NodeId
where
    T: Into<Value>,
{
    fn from(v: T) -> Self {
        Self::Value(v.into())
    }
}
