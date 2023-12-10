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

    fn lookup_path(&mut self, path: &Path) -> ValueRef<'expr>;
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

    pub fn resolve(&mut self, value: &'expr ValueExpr) -> ValueRef<'expr> {
        value.eval(self)
    }
}

impl<'a, 'expr> ValueResolver<'expr> for Deferred<'a, 'expr> {
    fn resolve_number(&mut self, value: &'expr ValueExpr) -> Option<Num> {
        match value.eval(self) {
            ValueRef::Owned(Owned::Num(num)) => Some(num),
            _ => None,
        }
    }

    fn resolve_bool(&mut self, value: &'expr ValueExpr) -> bool {
        value.eval(self).is_true()
    }

    fn resolve_path(&mut self, value: &'expr ValueExpr) -> Option<Path> {
        match value {
            ValueExpr::Ident(path) => Some(Path::from(&**path)),
            ValueExpr::Index(lhs, index) => {
                // lhs can only be either an ident or an index
                let lhs = self.resolve_path(lhs)?;
                let index = self.resolve_number(index)?.to_usize();
                Some(lhs.compose(index))
            }
            ValueExpr::Dot(lhs, rhs) => {
                let lhs = self.resolve_path(lhs)?;
                let rhs = self.resolve_path(rhs)?;
                Some(lhs.compose(rhs))
            }
            _ => None,
        }
    }

    fn lookup_path(&mut self, path: &Path) -> ValueRef<'expr> {
        match self.context.scopes.lookup(path) {
            ValueRef::Empty => ValueRef::Deferred(path.clone()),
            val => val,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Resolver -
// -----------------------------------------------------------------------------
/// Resolve the expression, including deferred values.
pub struct Resolver<'ctx, 'state> {
    context: &'ctx Context<'state, 'state>,
    node_id: Option<&'state NodeId>,
    is_deferred: bool,
}

impl<'ctx, 'state> Resolver<'ctx, 'state> {
    pub fn new(context: &'ctx Context<'state, 'state>, node_id: Option<&'state NodeId>) -> Self {
        Self {
            context,
            node_id,
            is_deferred: false,
        }
    }
}

impl<'state> Resolver<'_, 'state> {
    pub fn resolve(&mut self, value: &'state ValueExpr) -> ValueRef<'state> {
        match value.eval(self) {
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                self.context.state.get(&path, self.node_id)
            }
            val => val,
        }
    }

    pub fn is_deferred(&self) -> bool {
        self.is_deferred
    }

    pub fn resolve_string(&mut self, value: &'state ValueExpr) -> Option<String> {
        match value.eval(self) {
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
                match self.context.state.get(&path, self.node_id) {
                    ValueRef::Str(val) => Some(val.into()),
                    ValueRef::Owned(val) => Some(val.to_string()),
                    ValueRef::Empty => None,
                    val => {
                        // TODO: panic...
                        panic!("don't panic here: {val:?}")
                    }
                }
            }
            ValueRef::Empty => None,

            // TODO: probably shouldn't panic here, but we'll do it while working on this
            v => panic!("{v:?}"),
        }
    }

    pub fn resolve_list<T>(&mut self, value: &'state ValueExpr) -> SmallVec<[T; 4]>
    where
        T: for<'b> TryFrom<ValueRef<'b>>,
    {
        let mut output = SmallVec::<[T; 4]>::new();
        let value = value.eval(self);
        let value = match value {
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                self.context.state.get(&path, self.node_id)
            }
            val => val,
        };

        let mut resolver = Self::new(self.context, self.node_id);
        match value {
            ValueRef::Expressions(list) => {
                for expr in list {
                    let val = expr.eval(&mut resolver);
                    let Ok(val) = T::try_from(val) else { continue };
                    output.push(val);
                }

                if resolver.is_deferred {
                    self.is_deferred = true;
                }

                output
            }
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

impl<'state> ValueResolver<'state> for Resolver<'_, 'state> {
    fn resolve_number(&mut self, value: &'state ValueExpr) -> Option<Num> {
        match value.eval(self) {
            ValueRef::Owned(Owned::Num(num)) => Some(num),
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                match self.context.state.get(&path, self.node_id) {
                    ValueRef::Owned(Owned::Num(num)) => Some(num),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn resolve_bool(&mut self, value: &'state ValueExpr) -> bool {
        match value.eval(self) {
            ValueRef::Deferred(path) => {
                self.is_deferred = true;
                self.context.state.get(&path, self.node_id).is_true()
            }
            val => val.is_true(),
        }
    }

    fn resolve_path(&mut self, value: &'state ValueExpr) -> Option<Path> {
        match value {
            ValueExpr::Ident(path) => {
                let path = Path::from(&**path);
                match self.context.scopes.lookup(&path) {
                    ValueRef::Deferred(path) => Some(path),
                    ValueRef::Empty => Some(path),
                    val => {
                        panic!("this should never be anythign but a deferred path: {val:?}")
                    }
                }
            }
            ValueExpr::Index(lhs, index) => {
                // lhs can only be either an ident or an index
                let lhs = self.resolve_path(lhs)?;
                let index = self.resolve_number(index)?.to_usize();
                Some(lhs.compose(index))
            }
            ValueExpr::Dot(lhs, rhs) => {
                let lhs = self.resolve_path(lhs)?;
                let rhs = self.resolve_path(rhs)?;
                Some(lhs.compose(rhs))
            }
            _ => None,
        }
    }

    fn lookup_path(&mut self, path: &Path) -> ValueRef<'state> {
        match self.context.scopes.lookup(path) {
            ValueRef::Empty => {
                self.is_deferred = true;
                self.context.state.get(&path, self.node_id)
            }
            val => val,
        }
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

    // TODO: does the list and the hashmap even need to be RCd?
    List(Rc<[ValueExpr]>),
    Map(Rc<HashMap<String, ValueExpr>>),

    Add(Box<ValueExpr>, Box<ValueExpr>),
    Sub(Box<ValueExpr>, Box<ValueExpr>),
    Div(Box<ValueExpr>, Box<ValueExpr>),
    Mul(Box<ValueExpr>, Box<ValueExpr>),
    Mod(Box<ValueExpr>, Box<ValueExpr>),
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
        }
    }
}

macro_rules! none_to_empty {
    ($e:expr) => {
        match $e {
            Some(val) => val,
            None => return ValueRef::Empty,
        }
    };
}

impl ValueExpr {
    fn eval<'expr>(&'expr self, resolver: &mut impl ValueResolver<'expr>) -> ValueRef<'expr> {
        match self {
            Self::Owned(value) => ValueRef::Owned(*value),
            Self::String(value) => ValueRef::Str(&*value),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            Self::Add(lhs, rhs) => {
                let lhs = none_to_empty!(resolver.resolve_number(lhs));
                let rhs = none_to_empty!(resolver.resolve_number(rhs));
                ValueRef::Owned(Owned::Num(lhs + rhs))
            }
            Self::Sub(lhs, rhs) => {
                let lhs = none_to_empty!(resolver.resolve_number(lhs));
                let rhs = none_to_empty!(resolver.resolve_number(rhs));
                ValueRef::Owned(Owned::Num(lhs - rhs))
            }
            Self::Mul(lhs, rhs) => {
                let lhs = none_to_empty!(resolver.resolve_number(lhs));
                let rhs = none_to_empty!(resolver.resolve_number(rhs));
                ValueRef::Owned(Owned::Num(lhs * rhs))
            }
            Self::Mod(lhs, rhs) => {
                let lhs = none_to_empty!(resolver.resolve_number(lhs));
                let rhs = none_to_empty!(resolver.resolve_number(rhs));
                ValueRef::Owned(Owned::Num(lhs % rhs))
            }
            Self::Div(lhs, rhs) => {
                let lhs = none_to_empty!(resolver.resolve_number(lhs));
                let rhs = none_to_empty!(resolver.resolve_number(rhs));
                match !rhs.is_zero() {
                    true => ValueRef::Owned(Owned::Num(lhs / rhs)),
                    false => ValueRef::Empty,
                }
            }
            Self::Negative(expr) => {
                let num = none_to_empty!(resolver.resolve_number(expr));
                ValueRef::Owned(Owned::Num(num.to_negative()))
            }

            // -----------------------------------------------------------------------------
            //   - Conditions -
            // -----------------------------------------------------------------------------
            Self::Not(expr) => {
                let b = resolver.resolve_bool(expr);
                ValueRef::Owned((!b).into())
            }
            Self::Equality(lhs, rhs) => {
                let lhs = lhs.eval(resolver);
                let rhs = rhs.eval(resolver);
                ValueRef::Owned((lhs == rhs).into())
            }
            Self::Or(lhs, rhs) => {
                let lhs = lhs.eval(resolver);
                let rhs = rhs.eval(resolver);
                ValueRef::Owned((lhs.is_true() || rhs.is_true()).into())
            }
            Self::And(lhs, rhs) => {
                let lhs = lhs.eval(resolver);
                let rhs = rhs.eval(resolver);
                ValueRef::Owned((lhs.is_true() && rhs.is_true()).into())
            }

            // -----------------------------------------------------------------------------
            //   - Paths -
            // -----------------------------------------------------------------------------
            Self::Ident(path) => {
                let path = Path::from(&**path);
                resolver.lookup_path(&path)
            }
            Self::Dot(lhs, rhs) => {
                let lhs = none_to_empty!(resolver.resolve_path(lhs));
                let rhs = none_to_empty!(resolver.resolve_path(rhs));
                let path = lhs.compose(rhs);
                resolver.lookup_path(&path)
            }
            Self::Index(lhs, index) => {
                let lhs = none_to_empty!(resolver.resolve_path(lhs));
                let index = none_to_empty!(resolver.resolve_number(index)).to_usize();
                let path = lhs.compose(index);
                resolver.lookup_path(&path)
            }

            // -----------------------------------------------------------------------------
            //   - Collection -
            // -----------------------------------------------------------------------------
            Self::List(list) => ValueRef::Expressions(list),
            Self::Map(map) => ValueRef::ExpressionMap(map),
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

#[cfg(test)]
mod test {
    use crate::map::Map;
    use crate::testing::{
        add, and, div, dot, eq, ident, inum, list, modulo, mul, neg, not, or, strlit, sub, unum,
        TestExpression,
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

    #[test]
    fn sub_static() {
        let expr = sub(unum(10), unum(2));
        expr.test().expect_owned(8u8);
    }

    #[test]
    fn mul_static() {
        let expr = mul(unum(10), unum(2));
        expr.test().expect_owned(20u8);
    }

    #[test]
    fn div_static() {
        let expr = div(unum(10), unum(2));
        expr.test().expect_owned(5u8);
    }

    #[test]
    fn mod_static() {
        let expr = modulo(unum(5), unum(3));
        expr.test().expect_owned(2u8);
    }

    #[test]
    fn bools() {
        // false
        let expr = ident("is_false");
        expr.with_data([("is_false", false)]).expect_owned(false);

        // not is false
        let expr = not(ident("is_false"));
        expr.with_data([("is_false", false)]).expect_owned(true);

        // equality
        let expr = eq(ident("one"), ident("one"));
        expr.with_data([("one", 1)]).eval_bool(true);

        // not equality
        let expr = not(eq(ident("one"), ident("two")));
        expr.with_data([("one", 1), ("two", 2.into())])
            .eval_bool(true);

        // or
        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", true.into())])
            .eval_bool(true);

        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", false.into())])
            .eval_bool(true);

        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", false.into())])
            .eval_bool(false);

        // and
        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", true.into())])
            .eval_bool(true);

        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", true.into())])
            .eval_bool(false);

        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", false.into())])
            .eval_bool(false);
    }

    #[test]
    fn path() {
        let test = dot(ident("inner"), ident("name"))
            .with_data([("inner", Map::new([("name", "Fiddle McStick".to_string())]))]);
        let name = test.eval();
        assert!(matches!(name, ValueRef::Str("Fiddle McStick")));
    }

    #[test]
    fn string() {
        let expr = list(vec![strlit("Mr. "), dot(ident("inner"), ident("name"))]);
        let string = expr
            .with_data([("inner", Map::new([("name", "Fiddle McStick".to_string())]))])
            .eval_string()
            .unwrap();
        assert_eq!(string, "Mr. Fiddle McStick");
    }
}
