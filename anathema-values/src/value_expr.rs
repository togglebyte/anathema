use std::fmt::Display;
use std::rc::Rc;

use crate::{
    Collection, Context, NodeId, Num, Owned, Path, Scope, ScopeValue, State, Value, ValueRef,
};

pub enum OrPath<T> {
    Val(T),
    Path(Path),
    None,
}

// TODO: rename this to `Expression` and rename `compiler::Expression` to something else
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    Value(Value),

    Not(Box<ValueExpr>),
    Negative(Box<ValueExpr>),
    And(Box<ValueExpr>, Box<ValueExpr>),
    Or(Box<ValueExpr>, Box<ValueExpr>),

    Ident(Rc<str>),
    Dot(Box<ValueExpr>, Box<ValueExpr>),
    Index(Box<ValueExpr>, Box<ValueExpr>),

    List(Rc<[ValueExpr]>),
    Add(Box<ValueExpr>, Box<ValueExpr>),
    Sub(Box<ValueExpr>, Box<ValueExpr>),
    Div(Box<ValueExpr>, Box<ValueExpr>),
    Mul(Box<ValueExpr>, Box<ValueExpr>),
    Mod(Box<ValueExpr>, Box<ValueExpr>),

    Invalid,
}

impl Display for ValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(val) => write!(f, "{val}"),
            Self::Ident(s) => write!(f, "{s}"),
            Self::Index(lhs, idx) => write!(f, "{lhs}[{idx}]"),
            Self::Dot(lhs, rhs) => write!(f, "{lhs}.{rhs}"),
            Self::Not(expr) => write!(f, "!{expr}"),
            Self::Negative(expr) => write!(f, "-{expr}"),
            Self::Add(lhs, rhs) => write!(f, "{lhs} + {rhs}"),
            Self::Sub(lhs, rhs) => write!(f, "{lhs} - {rhs}"),
            Self::Mul(lhs, rhs) => write!(f, "{lhs} * {rhs}"),
            Self::Div(lhs, rhs) => write!(f, "{lhs} / {rhs}"),
            Self::Mod(lhs, rhs) => write!(f, "{lhs} % {rhs}"),
            Self::List(list) => {
                write!(
                    f,
                    "[{}]",
                    list.iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::And(lhs, rhs) => write!(f, "{lhs} && {rhs}"),
            Self::Or(lhs, rhs) => write!(f, "{lhs} || {rhs}"),
            Self::Invalid => write!(f, "<invalid>"),
        }
    }
}

impl<T> From<T> for ValueExpr
where
    T: Into<Value>,
{
    fn from(val: T) -> Self {
        Self::Value(val.into())
    }
}

impl ValueExpr {

    // Value from state = borrow
    // Value from expression = borrow
    // Value from scope = own

    pub fn eval_value(&self, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> () {
        match self {
            Self::Ident(path) => context.lookup(&path.into()),
            Self::Dot(lhs, rhs) => {
                let lhs = lhs.eval_path(context);
                let rhs = rhs.eval_path(context);
                let path = lhs.compose(rhs);
                context.lookup(&path)
            }
            Self::Index(lhs, index) => {
                let lhs = lhs.eval_path(context);
                let index = index.eval_num(context);
                let path = lhs.compose(index);
                context.lookup(&path)
            }
            Self::Value(val) => val,
            _ => Invalid
        }
    }

    // The context is required here:
    // for x in list
    //     text x + 1
    //
    // This has to resolve `x` as a scoped value,
    // and then evalute the expression x + 1
    pub fn value(&self, context: &Context<'_, '_>) -> OrPath<&Value> {
        match self {
            Self::Value(val) => OrPath::Val(val),
            Self::Ident(key) => OrPath::Path(Path::from(&**key)),
            // Self::Add(lhs, rhs) => eval_add(lhs, rhs, context),
            // Self::Sub(lhs, rhs) => eval_add(lhs, rhs, context),

            // a.b(1, 2, false)[1][2]

            // Self::Index(lhs, index) => OrPath::Path(Path::Index(&**key)),
            _ => {
                panic!()
            }
        }
    }

    pub fn list(&self) -> OrPath<Rc<[ValueExpr]>> {
        panic!()
    }

    pub fn eval<'val, T: 'val + ?Sized>(
        &'val self,
        context: &Context<'_, 'val>,
        node_id: Option<&NodeId>,
    ) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        match self {
            Self::Value(value) => value.try_into().ok(),
            expr @ (Self::Dot(..) | Self::Ident(_)) => {
                let path = eval_path(expr, context, node_id)?;
                context.get::<T>(&path, node_id)
            }
            _ => panic!(),
        }
    }

    pub fn eval_collection(
        &self,
        context: &Context<'_, '_>,
        node_id: Option<&NodeId>,
    ) -> Collection {
        panic!()
        // match self {
        //     Self::List(list) => Collection::Rc(list.clone()),
        //     _ => {

        //         let Some(path) = eval_path(self, context, node_id) else {
        //             return Collection::Empty;
        //         };

        //         context.resolve(path);
        //         panic!()
        //     }
        // }
    }
}

fn eval_path(
    expr: &ValueExpr,
    context: &Context<'_, '_>,
    node_id: Option<&NodeId>,
) -> Option<Path> {
    let path = match expr {
        ValueExpr::Ident(key) => Path::Key(key.to_string()),
        ValueExpr::Dot(lhs, rhs) => Path::Composite(
            eval_path(lhs, context, node_id)?.into(),
            eval_path(rhs, context, node_id)?.into(),
        ),
        ValueExpr::Index(lhs, index) => {
            let index = *index.eval::<u64>(context, node_id)?;
            let collection = eval_path(lhs, context, node_id)?;
            collection.compose(Path::Index(index as usize))
        }
        _ => return None,
    };

    Some(path)
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::ops::Deref;

    use super::*;
    use crate::testing::{unum, add, ident, TestState};
    use crate::{List, Scope, State, StateValue};

    #[test]
    fn test_add_dyn() {
        let state = TestState::new();
        let mut scope = Scope::new(None);
        let context = Context::new(&state, &mut scope);
        let expr = add(ident("counter"), unum(1));
        expr.eval_value(&context);

        // expr.eval_collection();
        // expr.eval_bool();
    }

    #[test]
    fn test_add_static() {
        // let expr = add(
        // assert_eq!(expected, actual);
    }
}
