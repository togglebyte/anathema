use std::fmt::Display;
use std::rc::Rc;

use smallvec::SmallVec;

use crate::hashmap::HashMap;
use crate::{Context, NodeId, Num, Owned, Path, ValueRef};

// -----------------------------------------------------------------------------
//   - Value resolver trait -
// -----------------------------------------------------------------------------
pub trait ValueResolver<'expr> {
    fn resolve_number(&mut self, value: &'expr ValueExpr) -> Option<Num>;

    fn resolve_bool(&mut self, value: &'expr ValueExpr) -> bool;

    fn resolve_path(&mut self, value: &'expr ValueExpr) -> Option<Path>;

    fn lookup_path(&mut self, path: &Path) -> Option<ValueRef<'expr>>;
}

// -----------------------------------------------------------------------------
//   - Deferred -
// -----------------------------------------------------------------------------
/// Only resolve up until a deferred path.
/// This means `ValueExpr::Deferred` will not be resolved, and instead returned.
pub struct Deferred<'a, 'expr> {
    context: &'a Context<'a, 'expr>,
}

impl<'a, 'expr> Deferred<'a, 'expr> {
    pub fn new(context: &'a Context<'a, 'expr>) -> Self {
        Self { context }
    }
}

impl<'a, 'expr> ValueResolver<'expr> for Deferred<'a, 'expr> {
    fn resolve_number(&mut self, value: &'expr ValueExpr) -> Option<Num> {
        match value.eval(self)? {
            ValueRef::Owned(Owned::Num(num)) => Some(num),
            _ => None,
        }
    }

    fn resolve_bool(&mut self, value: &'expr ValueExpr) -> bool {
        match value.eval(self) {
            Some(val) => val.is_true(),
            _ => false,
        }
    }

    fn resolve_path(&mut self, value: &'expr ValueExpr) -> Option<Path> {
        match value {
            ValueExpr::Ident(path) => Some(Path::from(&**path)),
            ValueExpr::Index(lhs, index) => {
                // lhs can only be either an ident or an index
                let lhs = self.resolve_path(lhs)?;
                let index = self.resolve_path(index)?;
                Some(lhs.compose(index))
            }
            _ => None,
        }
    }

    fn lookup_path(&mut self, path: &Path) -> Option<ValueRef<'expr>> {
        self.context.lookup(path)
    }
}

// -----------------------------------------------------------------------------
//   - Resolver -
// -----------------------------------------------------------------------------
/// Resolve the expression, including deferred values.
pub struct Resolver<'a, 'expr> {
    context: &'a Context<'a, 'expr>,
    node_id: Option<&'a NodeId>,
    is_deferred: bool,
}

impl<'a, 'expr> Resolver<'a, 'expr> {
    pub fn new(context: &'a Context<'a, 'expr>, node_id: Option<&'a NodeId>) -> Self {
        Self {
            context,
            node_id,
            is_deferred: false,
        }
    }

    pub fn is_deferred(&self) -> bool {
        self.is_deferred
    }

    pub fn resolve_string(&mut self, value: &'expr ValueExpr) -> Option<String> {
        match value.eval(self)? {
            ValueRef::Str(s) => Some(s.into()),
            ValueRef::Owned(s) => Some(s.to_string()),
            ValueRef::Expressions(list) => {
                let mut s = String::new();
                for expr in list {
                    let res = self.resolve_string(expr);
                    if let Some(res) = res {
                        s.push_str(&res);
                    }
                }
                Some(s)
            }
            ValueRef::ExpressionMap(map) => {
                panic!("how should this become a string");
            }
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                let p = path.to_string();
                match self.context.state.get(&path, self.node_id)? {
                    ValueRef::Str(val) => Some(val.into()),
                    ValueRef::Owned(val) => Some(val.to_string()),
                    val => {
                        // TODO: panic...
                        panic!("don't panic here: {val:?}")
                    }
                }
            }

            //     // TODO: probably shouldn't panic here, but we'll do it while working on this
            _ => panic!(),
        }
    }

    pub fn resolve_list<T>(&mut self, value: &'expr ValueExpr) -> SmallVec<[T; 4]>
    where
        T: for<'b> TryFrom<ValueRef<'b>>,
    {
        let mut output = SmallVec::<[T; 4]>::new();
        let Some(value) = value.eval(self) else {
            return output;
        };

        let value = match value {
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                match self.context.state.get(&path, self.node_id) {
                    Some(val) => val,
                    None => return output,
                }
            }
            val => val,
        };

        let mut resolver = Self::new(self.context, self.node_id);
        match value {
            ValueRef::Expressions(list) => {
                for expr in list {
                    let Some(val) = expr
                        .eval(&mut resolver)
                        .and_then(|val| T::try_from(val).ok())
                    else {
                        continue;
                    };
                    output.push(val);
                }
                if resolver.is_deferred {
                    self.is_deferred = true;
                }
                output
            },
            val => {
                let Ok(val) = T::try_from(val) else {
                    return output;
                };
                output.push(val);
                output
            }
        }
    }
}

impl<'a, 'expr> ValueResolver<'expr> for Resolver<'a, 'expr> {
    fn resolve_number(&mut self, value: &'expr ValueExpr) -> Option<Num> {
        match value.eval(self)? {
            ValueRef::Owned(Owned::Num(num)) => Some(num),
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                match self.context.state.get(&path, self.node_id)? {
                    ValueRef::Owned(Owned::Num(num)) => Some(num),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn resolve_bool(&mut self, value: &'expr ValueExpr) -> bool {
        match value.eval(self) {
            Some(ValueRef::Deferred(path)) => {
                self.is_deferred = true;
                match self.context.state.get(&path, self.node_id) {
                    Some(val) => val.is_true(),
                    _ => false,
                }
            }
            Some(val) => val.is_true(),
            _ => false,
        }
    }

    fn resolve_path(&mut self, value: &'expr ValueExpr) -> Option<Path> {
        match value {
            ValueExpr::Ident(path) => Some(Path::from(&**path)),
            ValueExpr::Index(lhs, index) => {
                // lhs can only be either an ident or an index
                let lhs = self.resolve_path(lhs)?;
                let index = self.resolve_path(index)?;
                Some(lhs.compose(index))
            }
            _ => None,
        }
    }

    fn lookup_path(&mut self, path: &Path) -> Option<ValueRef<'expr>> {
        self.context.lookup(path)
    }
}

// -----------------------------------------------------------------------------
//   - Value expressoin -
// -----------------------------------------------------------------------------
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

impl ValueExpr {
    pub fn eval<'e>(&'e self, resolver: &mut impl ValueResolver<'e>) -> Option<ValueRef<'e>> {
        match self {
            Self::Owned(value) => Some(ValueRef::Owned(*value)),
            Self::String(value) => Some(ValueRef::Str(&*value)),
            Self::Invalid => None,

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            Self::Add(lhs, rhs) => {
                let lhs = resolver.resolve_number(lhs)?;
                let rhs = resolver.resolve_number(rhs)?;
                Some(ValueRef::Owned(Owned::Num(lhs + rhs)))
            }
            Self::Sub(lhs, rhs) => {
                let lhs = resolver.resolve_number(lhs)?;
                let rhs = resolver.resolve_number(rhs)?;
                Some(ValueRef::Owned(Owned::Num(lhs - rhs)))
            }
            Self::Mul(lhs, rhs) => {
                let lhs = resolver.resolve_number(lhs)?;
                let rhs = resolver.resolve_number(rhs)?;
                Some(ValueRef::Owned(Owned::Num(lhs * rhs)))
            }
            Self::Mod(lhs, rhs) => {
                let lhs = resolver.resolve_number(lhs)?;
                let rhs = resolver.resolve_number(rhs)?;
                Some(ValueRef::Owned(Owned::Num(lhs % rhs)))
            }
            Self::Div(lhs, rhs) => {
                let lhs = resolver.resolve_number(lhs)?;
                let rhs = resolver.resolve_number(rhs)?;
                if rhs.is_zero() {
                    return None;
                }
                Some(ValueRef::Owned(Owned::Num(lhs / rhs)))
            }
            Self::Negative(expr) => {
                let num = resolver.resolve_number(expr)?;
                Some(ValueRef::Owned(Owned::Num(num.to_negative())))
            }

            // -----------------------------------------------------------------------------
            //   - Conditions -
            // -----------------------------------------------------------------------------
            Self::Not(expr) => {
                let b = resolver.resolve_bool(expr);
                Some(ValueRef::Owned((!b).into()))
            }
            Self::Equality(lhs, rhs) => {
                let lhs = lhs.eval(resolver)?;
                let rhs = rhs.eval(resolver)?;
                Some(ValueRef::Owned((lhs == rhs).into()))
            }
            Self::Or(lhs, rhs) => {
                let lhs = lhs.eval(resolver)?;
                let rhs = rhs.eval(resolver)?;
                Some(ValueRef::Owned((lhs.is_true() || rhs.is_true()).into()))
            }
            Self::And(lhs, rhs) => {
                let lhs = lhs.eval(resolver)?;
                let rhs = rhs.eval(resolver)?;
                Some(ValueRef::Owned((lhs.is_true() && rhs.is_true()).into()))
            }

            // -----------------------------------------------------------------------------
            //   - Paths -
            // -----------------------------------------------------------------------------
            Self::Ident(path) => {
                let value_ref = resolver.lookup_path(&Path::from(&**path))?;
                Some(value_ref)
            }
            Self::Dot(lhs, rhs) => {
                let lhs = resolver.resolve_path(lhs)?;
                let rhs = resolver.resolve_path(rhs)?;
                let path = lhs.compose(rhs);
                resolver.lookup_path(&path)
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

    // pub fn list_usize<P>(&self, context: &Context<'_, '_>) -> Vec<usize> {
    //     match self.eval_value_ref(context) {
    //         Some(ValueRef::Expressions(list)) => list
    //             .iter()
    //             .filter_map(|value_expr| value_expr.eval_number(context))
    //             .filter_map(|num| match num {
    //                 Num::Signed(val) => Some(val as usize),
    //                 Num::Unsigned(val) => Some(val as usize),
    //                 Num::Float(_) => None,
    //             })
    //             .collect(),
    //         _ => vec![],
    //     }
    // }
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

#[cfg(test)]
mod test {
    use crate::map::Map;
    use crate::testing::{
        add, and, div, dot, eq, ident, inum, list, modulo, mul, neg, not, or, strlit, sub, unum, TestExpression,
    };
    use crate::ValueRef;

    #[test]
    fn add_dyn() {
        let expr = add(neg(inum(1)), neg(unum(2)));
        expr.with_data([("counter", 2usize)]).expect_owned(-3);
    }

    #[test]
    fn add_static() {
        let expr = add(neg(inum(1)), neg(unum(2)));
        expr.test().expect_owned(-3);
    }

//     #[test]
//     fn sub_static() {
//         let expr = sub(unum(10), unum(2));
//         expr.test([]).expect_owned(8u8);
//     }

//     #[test]
//     fn mul_static() {
//         let expr = mul(unum(10), unum(2));
//         expr.test([]).expect_owned(20u8);
//     }

//     #[test]
//     fn div_static() {
//         let expr = div(unum(10), unum(2));
//         expr.test([]).expect_owned(5u8);
//     }

//     #[test]
//     fn mod_static() {
//         panic!()
//         // let expr = modulo(unum(5), unum(3));
//         // expr.test([]).expect_owned(2u8);
//     }

//     #[test]
//     fn bools() {
//         // false
//         let expr = ident("is_false");
//         expr.test([("is_false", &false)]).expect_owned(false);

//         // // not is false
//         // let expr = not(ident("is_false"));
//         // expr.test([("is_false", false.into())]).expect_owned(true);

//         // // equality
//         // let expr = eq(ident("one"), ident("one"));
//         // expr.test([("one", 1.into())]).expect_owned(true);

//         // // not equality
//         // let expr = not(eq(ident("one"), ident("two")));
//         // expr.test([("one", 1.into()), ("two", 2.into())])
//         //     .expect_owned(true);

//         // // or
//         // let expr = or(ident("one"), ident("two"));
//         // expr.test([("one", false.into()), ("two", true.into())])
//         //     .expect_owned(true);

//         // let expr = or(ident("one"), ident("two"));
//         // expr.test([("one", true.into()), ("two", false.into())])
//         //     .expect_owned(true);

//         // let expr = or(ident("one"), ident("two"));
//         // expr.test([("one", false.into()), ("two", false.into())])
//         //     .expect_owned(false);

//         // // and
//         // let expr = and(ident("one"), ident("two"));
//         // expr.test([("one", true.into()), ("two", true.into())])
//         //     .expect_owned(true);

//         // let expr = and(ident("one"), ident("two"));
//         // expr.test([("one", false.into()), ("two", true.into())])
//         //     .expect_owned(false);

//         // let expr = and(ident("one"), ident("two"));
//         // expr.test([("one", true.into()), ("two", false.into())])
//         //     .expect_owned(false);
//     }

//     #[test]
//     fn path() {
//         panic!()
//         // let test = dot(ident("inner"), ident("name")).test([]);
//         // let name = test.eval().unwrap();
//         // assert!(matches!(name, ValueRef::Str("Fiddle McStick")));
//     }

//     #[test]
//     fn string() {
//         let expr = list(vec![strlit("Mr. "), dot(ident("inner"), ident("name"))]);
//         // let string = expr.test(]).eval_string().unwrap();
//         // assert_eq!(string, "Mr. Fiddle McStick");
//     }
}
