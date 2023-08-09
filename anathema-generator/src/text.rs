use anathema_values::{PathId, ValueRef};

// -----------------------------------------------------------------------------
//   - Expressions -
// -----------------------------------------------------------------------------
pub enum TextExpr {
    String(String),
    Fragments(Vec<FragmentExpr>),
}

pub enum FragmentExpr {
    String(String),
    Path(PathId),
}

// -----------------------------------------------------------------------------
//   - Values -
// -----------------------------------------------------------------------------
pub enum Text<T> {
    String(String),
    Fragments(Vec<Fragment<T>>),
}

pub enum Fragment<T> {
    String(String),
    Path(ValueRef<T>),
}

