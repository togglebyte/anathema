use std::fmt::Display;
use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::{Context, NodeId, Num, Owned, Path, Value, ValueRef};

// TODO: rename this to `Expression` and rename `compiler::Expression` to something else
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    Owned(Owned),
    String(Rc<str>),

    Not(Box<ValueExpr>),
    Negative(Box<ValueExpr>),
    And(Box<ValueExpr>, Box<ValueExpr>),
    Or(Box<ValueExpr>, Box<ValueExpr>),
    Equality(Box<ValueExpr>, Box<ValueExpr>),

    Ident(Rc<str>),
    Dot(Box<ValueExpr>, Box<ValueExpr>),
    Index(Box<ValueExpr>, Box<ValueExpr>),

    List(Rc<[ValueExpr]>),
    Map(Rc<HashMap<String, ValueExpr>>),

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
            Self::Owned(val) => write!(f, "{val}"),
            Self::String(val) => write!(f, "{val}"),
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
            Self::Map(map) => {
                write!(
                    f,
                    "{{{}}}",
                    map.iter()
                        .map(|(key, val)| format!("{key}: {val}"))
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

impl From<Box<ValueExpr>> for ValueExpr {
    fn from(val: Box<ValueExpr>) -> Self {
        *val
    }
}

impl<T> From<T> for ValueExpr
where
    T: Into<Owned>,
{
    fn from(val: T) -> Self {
        Self::Owned(val.into())
    }
}

impl From<String> for ValueExpr {
    fn from(val: String) -> Self {
        Self::String(val.into())
    }
}

impl From<&str> for ValueExpr {
    fn from(val: &str) -> Self {
        Self::String(val.into())
    }
}

impl ValueExpr {
    pub fn resolve<T>(&self, context: &Context<'_, '_>, node_id: Option<&NodeId>) -> Value<T>
    where
        for<'b> T: TryFrom<ValueRef<'b>>,
    {
        match self.eval_value_ref(context) {
            Some(ValueRef::Deferred(path)) => {
                let val = context.state.get(&path, node_id);
                let val = val.and_then(|val_ref| T::try_from(val_ref).ok());
                Value::Cached {
                    val,
                    path: path.clone(),
                }
            }
            Some(val) => match T::try_from(val) {
                Ok(val) => Value::Static(val),
                Err(_) => Value::Empty,
            },
            None => Value::Empty,
        }
    }

    pub fn eval_bool(&self, context: &Context<'_, '_>) -> bool {
        panic!("come back to this, we need to deal with deferred values for control flow");
        // match self.eval_value_ref(context, node_id) {
        //     Some(value) => value.is_true(),
        //     _ => false,
        // }
    }

    fn eval_number(&self, context: &Context<'_, '_>) -> Option<Num> {
        match self.eval_value_ref(context)? {
            ValueRef::Owned(Owned::Num(num)) => Some(num),
            _ => None,
        }
    }

    pub fn eval_path(&self, context: &Context<'_, '_>) -> Option<Path> {
        match self {
            Self::Ident(path) => Some(Path::from(&**path)),
            Self::Index(lhs, index) => {
                // lhs can only be either an ident or an index
                let lhs = lhs.eval_path(context)?;
                let index = index.eval_path(context)?;
                Some(lhs.compose(index))
            }
            _ => None,
        }
    }

    //     pub fn eval_string(
    //         &self,
    //         context: &Context<'_, '_>,
    //         node_id: Option<&NodeId>,
    //     ) -> Option<String> {
    //         match self.eval_value_ref(context)? {
    //             ValueRef::Str(s) => Some(s.into()),
    //             ValueRef::Owned(s) => Some(s.to_string()),
    //             ValueRef::Expressions(list) => {
    //                 let mut s = String::new();
    //                 for expr in list {
    //                     let res = expr.eval_string(context, node_id);
    //                     if let Some(res) = res {
    //                         s.push_str(&res);
    //                     }
    //                 }
    //                 Some(s)
    //             }
    //             // TODO: probably shouldn't panic here, but we'll do it while working on this
    //             _ => panic!(),
    //         }
    //     }

    pub fn list_usize<P>(&self, context: &Context<'_, '_>) -> Vec<usize> {
        match self.eval_value_ref(context) {
            Some(ValueRef::Expressions(list)) => list
                .iter()
                .filter_map(|value_expr| value_expr.eval_number(context))
                .filter_map(|num| match num {
                    Num::Signed(val) => Some(val as usize),
                    Num::Unsigned(val) => Some(val as usize),
                    Num::Float(_) => None,
                })
                .collect(),
            _ => vec![],
        }
    }

    pub fn eval_value_ref<'expr>(&'expr self, context: &Context<'_, 'expr>) -> Option<ValueRef<'expr>> {
        match self {
            Self::Owned(value) => Some(ValueRef::Owned(*value)),
            Self::String(value) => Some(ValueRef::Str(&*value)),
            Self::Invalid => None,

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            Self::Add(lhs, rhs) => {
                let lhs = lhs.eval_number(context)?;
                let rhs = rhs.eval_number(context)?;
                Some(ValueRef::Owned(Owned::Num(lhs + rhs)))
            }
            Self::Sub(lhs, rhs) => {
                let lhs = lhs.eval_number(context)?;
                let rhs = rhs.eval_number(context)?;
                Some(ValueRef::Owned(Owned::Num(lhs - rhs)))
            }
            Self::Mul(lhs, rhs) => {
                let lhs = lhs.eval_number(context)?;
                let rhs = rhs.eval_number(context)?;
                Some(ValueRef::Owned(Owned::Num(lhs * rhs)))
            }
            Self::Mod(lhs, rhs) => {
                let lhs = lhs.eval_number(context)?;
                let rhs = rhs.eval_number(context)?;
                Some(ValueRef::Owned(Owned::Num(lhs % rhs)))
            }
            Self::Div(lhs, rhs) => {
                let lhs = lhs.eval_number(context)?;
                let rhs = rhs.eval_number(context)?;
                if rhs.is_zero() {
                    return None;
                }
                Some(ValueRef::Owned(Owned::Num(lhs / rhs)))
            }
            Self::Negative(expr) => {
                let num = expr.eval_number(context)?;
                Some(ValueRef::Owned(Owned::Num(num.to_negative())))
            }

            // -----------------------------------------------------------------------------
            //   - Conditions -
            // -----------------------------------------------------------------------------
            Self::Not(expr) => {
                let b = expr.eval_bool(context);
                Some(ValueRef::Owned((!b).into()))
            }
            Self::Equality(lhs, rhs) => {
                let lhs = lhs.eval_value_ref(context)?;
                let rhs = rhs.eval_value_ref(context)?;
                Some(ValueRef::Owned((lhs == rhs).into()))
            }
            Self::Or(lhs, rhs) => {
                let lhs = lhs.eval_value_ref(context)?;
                let rhs = rhs.eval_value_ref(context)?;
                Some(ValueRef::Owned((lhs.is_true() || rhs.is_true()).into()))
            }
            Self::And(lhs, rhs) => {
                let lhs = lhs.eval_value_ref(context)?;
                let rhs = rhs.eval_value_ref(context)?;
                Some(ValueRef::Owned((lhs.is_true() && rhs.is_true()).into()))
            }

            // -----------------------------------------------------------------------------
            //   - Paths -
            // -----------------------------------------------------------------------------
            Self::Ident(path) => context.lookup(&Path::from(&**path)),
            Self::Dot(lhs, rhs) => {
                let lhs = lhs.eval_path(context)?;
                let rhs = rhs.eval_path(context)?;
                let path = lhs.compose(rhs);
                context.lookup(&path)
            }
            Self::Index(_lhs, _index) => {
                // TODO: index lookup
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
            Self::Map(map) => Some(ValueRef::ExpressionMap(map)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::testing::{
        add, and, div, dot, eq, ident, inum, list, modulo, mul, neg, not, or, strlit, sub, unum,
    };
    use crate::ValueRef;

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
        // false
        let expr = ident("is_false");
        expr.test([("is_false", false.into())]).expect_owned(false);

        // not is false
        let expr = not(ident("is_false"));
        expr.test([("is_false", false.into())]).expect_owned(true);

        // equality
        let expr = eq(ident("one"), ident("one"));
        expr.test([("one", 1.into())]).expect_owned(true);

        // not equality
        let expr = not(eq(ident("one"), ident("two")));
        expr.test([("one", 1.into()), ("two", 2.into())])
            .expect_owned(true);

        // or
        let expr = or(ident("one"), ident("two"));
        expr.test([("one", false.into()), ("two", true.into())])
            .expect_owned(true);

        let expr = or(ident("one"), ident("two"));
        expr.test([("one", true.into()), ("two", false.into())])
            .expect_owned(true);

        let expr = or(ident("one"), ident("two"));
        expr.test([("one", false.into()), ("two", false.into())])
            .expect_owned(false);

        // and
        let expr = and(ident("one"), ident("two"));
        expr.test([("one", true.into()), ("two", true.into())])
            .expect_owned(true);

        let expr = and(ident("one"), ident("two"));
        expr.test([("one", false.into()), ("two", true.into())])
            .expect_owned(false);

        let expr = and(ident("one"), ident("two"));
        expr.test([("one", true.into()), ("two", false.into())])
            .expect_owned(false);
    }

    #[test]
    fn path() {
        let test = dot(ident("inner"), ident("name")).test([]);
        let name = test.eval().unwrap();
        assert!(matches!(name, ValueRef::Str("Fiddle McStick")));
    }

    #[test]
    fn string() {
        let expr = list(vec![strlit("Mr. "), dot(ident("inner"), ident("name"))]);
        let string = expr.test([]).eval_string().unwrap();
        assert_eq!(string, "Mr. Fiddle McStick");
    }
}
