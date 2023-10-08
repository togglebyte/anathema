use std::fmt::Display;
use std::rc::Rc;

use crate::{Collection, Context, NodeId, Num, Owned, Path, Value, ValueRef};

// TODO: rename this to `Expression` and rename `compiler::Expression` to something else
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    Value(Value),

    Not(Box<ValueExpr>),
    Negative(Box<ValueExpr>),
    And(Box<ValueExpr>, Box<ValueExpr>),
    Or(Box<ValueExpr>, Box<ValueExpr>),
    Equality(Box<ValueExpr>, Box<ValueExpr>),

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
            Self::Equality(lhs, rhs) => write!(f, "{lhs} == {rhs}"),
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
    fn eval_bool(&self, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> bool {
        match self.eval_value(context, node_id) {
            Some(ValueRef::Owned(Owned::Bool(true))) => true,
            _ => false,
        }
    }

    fn eval_number(&self, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> Option<Num> {
        match self.eval_value(context, node_id)? {
            ValueRef::Owned(Owned::Num(num)) => Some(num),
            _ => None,
        }
    }

    fn eval_path(&self, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> Option<Path> {
        match self {
            Self::Ident(path) => Some(Path::from(&**path)),
            Self::Index(lhs, index) => {
                // lhs can only be either an ident or an index
                let lhs = lhs.eval_path(context, node_id)?;
                let index = index.eval_path(context, node_id)?;
                Some(lhs.compose(index))
            }
            _ => None,
        }
    }

    pub fn eval_value<'a, 'val>(
        &'a self,
        context: &Context<'a, 'val>,
        node_id: Option<&NodeId>,
    ) -> Option<ValueRef<'_>> {
        match self {
            Self::Value(Value::Owned(value)) => Some(ValueRef::Owned(*value)),
            Self::Not(expr) => {
                let b = expr.eval_bool(context, node_id);
                Some(ValueRef::Owned((!b).into()))
            }
            Self::Equality(lhs, rhs) => {
                let lhs = lhs.eval_value(context, node_id)?;
                let rhs = rhs.eval_value(context, node_id)?;
                Some(ValueRef::Owned((lhs == rhs).into()))
            }

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            Self::Add(lhs, rhs) => {
                let lhs = lhs.eval_number(context, node_id)?;
                let rhs = rhs.eval_number(context, node_id)?;
                Some(ValueRef::Owned(Owned::Num(lhs + rhs)))
            }
            Self::Sub(lhs, rhs) => {
                let lhs = lhs.eval_number(context, node_id)?;
                let rhs = rhs.eval_number(context, node_id)?;
                Some(ValueRef::Owned(Owned::Num(lhs - rhs)))
            }
            Self::Mul(lhs, rhs) => {
                let lhs = lhs.eval_number(context, node_id)?;
                let rhs = rhs.eval_number(context, node_id)?;
                Some(ValueRef::Owned(Owned::Num(lhs * rhs)))
            }
            Self::Mod(lhs, rhs) => {
                let lhs = lhs.eval_number(context, node_id)?;
                let rhs = rhs.eval_number(context, node_id)?;
                Some(ValueRef::Owned(Owned::Num(lhs % rhs)))
            }
            Self::Div(lhs, rhs) => {
                let lhs = lhs.eval_number(context, node_id)?;
                let rhs = rhs.eval_number(context, node_id)?;
                if rhs.is_zero() {
                    return None;
                }
                Some(ValueRef::Owned(Owned::Num(lhs / rhs)))
            }
            Self::Negative(expr) => {
                let num = expr.eval_number(context, node_id)?;
                Some(ValueRef::Owned(Owned::Num(num.to_negative())))
            }

            // -----------------------------------------------------------------------------
            //   - Paths -
            // -----------------------------------------------------------------------------
            Self::Ident(path) => context.lookup(&Path::from(&**path), node_id),
            Self::Dot(lhs, rhs) => {
                let lhs = lhs.eval_path(context, node_id)?;
                let rhs = rhs.eval_path(context, node_id)?;
                let path = lhs.compose(rhs);
                context.lookup(&path, node_id)
            }
            Self::Index(lhs, index) => {
                panic!("not quite there...");
                // let lhs = lhs.eval_path(context);
                // let index = index.eval_num(context);
                // let path = lhs.compose(index);
                // context.lookup(&path)
            }

            // -----------------------------------------------------------------------------
            //   - Collection -
            // -----------------------------------------------------------------------------
            Self::List(list) => Some(ValueRef::Expressions(list)),
            _ => panic!(),
        }
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
                let path = expr.eval_path(context, node_id)?;
                context.get::<T>(&path, node_id)
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::ops::Deref;

    use super::*;
    use crate::testing::{
        add, boolean, div, dot, eq, ident, inum, modulo, mul, neg, not, sub, unum, TestState,
    };
    use crate::{List, Scope, State, StateValue};

    #[test]
    fn add_dyn() {
        let expr = add(neg(inum(1)), neg(unum(2)));
        expr.test([("counter", 2.into())]).expect_owned(-3);
    }

    #[test]
    fn add_static() {
        let expr = add(neg(inum(1)), neg(unum(2)));
        expr.test([]).expect_owned(-3);
    }

    #[test]
    fn sub_static() {
        let expr = sub(unum(10), unum(2));
        expr.test([]).expect_owned(8u8);
    }

    #[test]
    fn mul_static() {
        let expr = mul(unum(10), unum(2));
        expr.test([]).expect_owned(20u8);
    }

    #[test]
    fn div_static() {
        let expr = div(unum(10), unum(2));
        expr.test([]).expect_owned(5u8);
    }

    #[test]
    fn mod_static() {
        let expr = modulo(unum(5), unum(3));
        expr.test([]).expect_owned(2u8);
    }

    #[test]
    fn bools() {
        let expr = ident("is_false");
        expr.test([("is_false", false.into())]).expect_owned(false);

        let expr = not(ident("is_false"));
        expr.test([("is_false", false.into())]).expect_owned(true);

        let expr = eq(ident("one"), ident("one"));
        expr.test([("one", 1.into())]).expect_owned(true);

        let expr = not(eq(ident("one"), ident("two")));
        expr.test([("one", 1.into()), ("two", 2.into())])
            .expect_owned(true);
    }

    #[test]
    fn path() {
        let mut expr = dot(ident("a"), ident("b")).test([]);
        // expr.scope.scope();
        // let x = expr.eval();
        // panic!("{x:?}");
    }
}
