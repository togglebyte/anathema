use std::fmt::Display;
use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::value::{ExpressionMap, Expressions};
use crate::{Context, NodeId, Owned, Path, ScopeValue, ValueRef};

// -----------------------------------------------------------------------------
//   - Value resolver trait -
// -----------------------------------------------------------------------------
pub trait Resolver<'expr> {
    fn resolve(&mut self, path: &Path) -> ValueRef<'expr>;

    fn lookup(&mut self, ident: &str) -> ValueRef<'expr>;

    fn resolve_list_lookup(&mut self, list: &'expr ValueExpr, index: usize) -> ValueRef<'expr>;

    fn resolve_map_lookup(&mut self, map: &'expr ValueExpr, ident: &str) -> ValueRef<'expr>;
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

impl<'a, 'expr> Resolver<'expr> for Deferred<'a, 'expr> {
    fn resolve(&mut self, path: &Path) -> ValueRef<'expr> {
        match self.context.scopes.lookup(path) {
            None => ValueRef::Deferred,
            Some(ScopeValue::Value(value)) => *value,
            Some(ScopeValue::Deferred(..)) => {
                panic!("not sure what to do here yet, can this even happen?")
            }
            Some(ScopeValue::DeferredList(..)) => {
                panic!("not sure what to do here yet, can this even happen?")
            }
        }
    }

    fn lookup(&mut self, ident: &str) -> ValueRef<'expr> {
        let val = self.context.scopes.lookup(&(ident.into()));
        match val {
            Some(ScopeValue::Value(value)) => *value,
            _ => ValueRef::Deferred,
        }
    }

    fn resolve_list_lookup(&mut self, list: &'expr ValueExpr, index: usize) -> ValueRef<'expr> {
        match list.eval(self) {
            ValueRef::Expressions(list) => list
                .get(index)
                .map(|expr| expr.eval(self))
                .unwrap_or(ValueRef::Empty),
            ValueRef::Deferred => ValueRef::Deferred,
            _ => ValueRef::Empty,
        }
    }

    fn resolve_map_lookup(&mut self, map: &'expr ValueExpr, ident: &str) -> ValueRef<'expr> {
        match map.eval(self) {
            ValueRef::ExpressionMap(map) => map
                .get(ident)
                .map(|expr| expr.eval(self))
                .unwrap_or(ValueRef::Empty),
            ValueRef::Deferred => ValueRef::Deferred,
            _ => ValueRef::Empty,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Resolver -
//   This should never return a deferred value, instead
//   it should resolve any deferred value before returning
//
//   The immediate resolver is the only resolver that will
//   access the state, therefore no other resolver needs a NodeId
// -----------------------------------------------------------------------------
/// Resolve the expression, including deferred values.
pub struct Immediate<'ctx, 'state> {
    context: &'ctx Context<'state, 'state>,
    node_id: &'state NodeId,
    is_deferred: bool,
}

impl<'ctx, 'state> Immediate<'ctx, 'state> {
    pub fn new(context: &'ctx Context<'state, 'state>, node_id: &'state NodeId) -> Self {
        Self {
            context,
            node_id,
            is_deferred: false,
        }
    }
}

impl<'state> Immediate<'_, 'state> {
    pub fn is_deferred(&self) -> bool {
        self.is_deferred
    }
}

impl<'state> Resolver<'state> for Immediate<'_, 'state> {
    fn resolve(&mut self, path: &Path) -> ValueRef<'state> {
        match self.context.scopes.lookup(path) {
            Some(ScopeValue::Value(value)) => *value,
            Some(ScopeValue::Deferred(expr)) => {
                self.is_deferred = true;
                expr.eval(self)
            }
            Some(&ScopeValue::DeferredList(index, expr)) => {
                self.is_deferred = true;
                match expr.eval(self) {
                    ValueRef::List(list) => {
                        let path = index.into();
                        list.state_get(&path, self.node_id)
                    }
                    // TODO: this might be unreachable, investimagate!
                    _ => panic!(),
                }
            }
            None => {
                //
                match self
                    .context
                    .internal_state
                    .map(|s| s.state_get(path, self.node_id))
                {
                    Some(ValueRef::Empty) | None => {
                        match self.context.state.state_get(path, self.node_id) {
                            ValueRef::Empty => match self.context.meta {
                                Some(meta) => match meta.state_get(path, self.node_id) {
                                    ValueRef::Empty => ValueRef::Empty,
                                    val => {
                                        self.is_deferred = true;
                                        val
                                    }
                                },
                                None => ValueRef::Empty,
                            },
                            val => {
                                self.is_deferred = true;
                                val
                            }
                        }
                    }
                    Some(val) => match val {
                        ValueRef::Empty => ValueRef::Empty,
                        val => {
                            self.is_deferred = true;
                            val
                        }
                    },
                }
            }
        }
    }

    fn lookup(&mut self, ident: &str) -> ValueRef<'state> {
        let path = ident.into();
        self.resolve(&path)
    }

    fn resolve_list_lookup(&mut self, list: &'state ValueExpr, index: usize) -> ValueRef<'state> {
        match list.eval(self) {
            ValueRef::List(list) => {
                let index = index.into();
                list.state_get(&index, self.node_id)
            }
            ValueRef::Expressions(list) => {
                let value_expr = match list.get(index) {
                    None => return ValueRef::Empty,
                    Some(expr) => expr,
                };
                value_expr.eval(self)
            }
            _ => ValueRef::Empty,
        }
    }

    fn resolve_map_lookup(&mut self, map: &'state ValueExpr, ident: &str) -> ValueRef<'state> {
        match map.eval(self) {
            ValueRef::Map(map) => {
                let ident = ident.into();
                map.state_get(&ident, self.node_id)
            }
            ValueRef::ExpressionMap(map) => {
                let value_expr = match map.get(ident) {
                    None => return ValueRef::Empty,
                    Some(expr) => expr,
                };
                value_expr.eval(self)
            }
            _ => ValueRef::Empty,
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
    Greater(Box<ValueExpr>, Box<ValueExpr>),
    GreaterEqual(Box<ValueExpr>, Box<ValueExpr>),
    Less(Box<ValueExpr>, Box<ValueExpr>),
    LessEqual(Box<ValueExpr>, Box<ValueExpr>),

    Ident(Rc<str>),
    Dot(Box<ValueExpr>, Box<ValueExpr>),
    Index(Box<ValueExpr>, Box<ValueExpr>),

    // List and Map are both Rc'd as expressions are
    // cloned for `Value<T>` and a few other places.
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
            Self::Greater(lhs, rhs) => write!(f, "{lhs} > {rhs}"),
            Self::GreaterEqual(lhs, rhs) => write!(f, "{lhs} >= {rhs}"),
            Self::Less(lhs, rhs) => write!(f, "{lhs} < {rhs}"),
            Self::LessEqual(lhs, rhs) => write!(f, "{lhs} <= {rhs}"),
        }
    }
}

macro_rules! eval_num {
    ($e:expr, $resolver:expr) => {
        match $e.eval($resolver) {
            ValueRef::Owned(Owned::Num(num)) => num,
            ValueRef::Deferred => return ValueRef::Deferred,
            _ => return ValueRef::Empty,
        }
    };
}

impl ValueExpr {
    fn eval_list_lookup<'expr>(
        &'expr self,
        resolver: &mut impl Resolver<'expr>,
        index: &'expr ValueExpr,
    ) -> ValueRef<'expr> {
        let index = match index.eval(resolver) {
            ValueRef::Owned(Owned::Num(n)) => n.to_usize(),
            ValueRef::Deferred => return ValueRef::Deferred,
            _ => return ValueRef::Empty,
        };

        resolver.resolve_list_lookup(self, index)
    }

    fn eval_map_lookup<'expr>(
        &'expr self,
        resolver: &mut impl Resolver<'expr>,
        ident: &'expr ValueExpr,
    ) -> ValueRef<'expr> {
        let ident = match ident {
            ValueExpr::Ident(ident) => ident,
            _ => return ValueRef::Empty,
        };

        resolver.resolve_map_lookup(self, ident)
    }

    pub fn eval_string<'expr>(&'expr self, resolver: &mut impl Resolver<'expr>) -> Option<String> {
        match self.eval(resolver) {
            ValueRef::Str(s) => Some(s.into()),
            ValueRef::Owned(s) => Some(s.to_string()),
            ValueRef::Expressions(Expressions(list)) => {
                let mut s = String::new();
                for expr in list {
                    let res = expr.eval_string(resolver);
                    if let Some(res) = res {
                        s.push_str(&res);
                    }
                }
                Some(s)
            }
            _ => None,
        }
    }

    pub fn eval_vec<'expr>(
        &'expr self,
        resolver: &mut impl Resolver<'expr>,
    ) -> Option<Vec<ValueRef<'_>>> {
        match self.eval(resolver) {
            ValueRef::Expressions(Expressions(list)) => {
                let mut v = Vec::with_capacity(list.len());
                for expr in list {
                    let res = expr.eval(resolver);
                    v.push(res);
                }
                Some(v)
            }
            _ => None,
        }
    }

    // Even though the lifetime is named `'expr`, the value isn't necessarily tied to an expression.
    //
    // Static values originate from expressions and will have the aforementioned lifetime,
    // however a value could also stem from a state (by resolving a deferred value).
    // A value that originates from `State` can only live for the duration of the layout phase.
    pub fn eval<'expr>(&'expr self, resolver: &mut impl Resolver<'expr>) -> ValueRef<'expr> {
        match self {
            Self::Owned(value) => ValueRef::Owned(*value),
            Self::String(value) => ValueRef::Str(value),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            op @ (Self::Add(lhs, rhs)
            | Self::Sub(lhs, rhs)
            | Self::Mul(lhs, rhs)
            | Self::Mod(lhs, rhs)
            | Self::Div(lhs, rhs)
            | Self::Greater(lhs, rhs)
            | Self::GreaterEqual(lhs, rhs)
            | Self::Less(lhs, rhs)
            | Self::LessEqual(lhs, rhs)) => {
                let lhs = eval_num!(lhs, resolver);
                let rhs = eval_num!(rhs, resolver);

                match op {
                    Self::Add(..) => ValueRef::Owned(Owned::Num(lhs + rhs)),
                    Self::Sub(..) => ValueRef::Owned(Owned::Num(lhs - rhs)),
                    Self::Mul(..) => ValueRef::Owned(Owned::Num(lhs * rhs)),
                    Self::Mod(..) => ValueRef::Owned(Owned::Num(lhs % rhs)),
                    Self::Div(..) if !rhs.is_zero() => ValueRef::Owned(Owned::Num(lhs / rhs)),
                    Self::Div(..) => ValueRef::Empty,
                    Self::Greater(..) => {
                        ValueRef::Owned(Owned::Bool(lhs.to_u128() > rhs.to_u128()))
                    }
                    Self::GreaterEqual(..) => {
                        ValueRef::Owned(Owned::Bool(lhs.to_u128() >= rhs.to_u128()))
                    }
                    Self::Less(..) => ValueRef::Owned(Owned::Bool(lhs.to_u128() < rhs.to_u128())),
                    Self::LessEqual(..) => {
                        ValueRef::Owned(Owned::Bool(lhs.to_u128() <= rhs.to_u128()))
                    }
                    _ => unreachable!(),
                }
            }

            Self::Negative(expr) => {
                let num = eval_num!(expr, resolver);
                ValueRef::Owned(Owned::Num(num.to_negative()))
            }

            // -----------------------------------------------------------------------------
            //   - Conditions -
            // -----------------------------------------------------------------------------
            Self::Not(expr) => {
                let b = expr.eval(resolver).is_true();
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
            Self::Ident(ident) => resolver.lookup(ident),
            Self::Index(lhs, index) => lhs.eval_list_lookup(resolver, index),
            Self::Dot(lhs, rhs) => lhs.eval_map_lookup(resolver, rhs),

            // -----------------------------------------------------------------------------
            //   - Collection -
            // -----------------------------------------------------------------------------
            Self::List(list) => ValueRef::Expressions(Expressions::new(list)),
            Self::Map(map) => ValueRef::ExpressionMap(ExpressionMap::new(map)),
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
        add, and, div, dot, eq, greater_than, greater_than_equal, ident, inum, less_than,
        less_than_equal, list, modulo, mul, neg, not, or, strlit, sub, unum,
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
    fn greater_than_static() {
        let expr = greater_than(unum(5), unum(3));
        expr.test().expect_owned(true);
    }

    #[test]
    fn greater_than_equal_static() {
        let expr = greater_than_equal(unum(5), unum(3));
        expr.test().expect_owned(true);

        let expr = greater_than_equal(unum(3), unum(3));
        expr.test().expect_owned(true);
    }

    #[test]
    fn greater_than_equal_dynamic() {
        let expr = greater_than_equal(unum(5), ident("counter"));
        expr.with_data([("counter", 3)]).expect_owned(true);

        let expr = greater_than_equal(unum(5), ident("counter"));
        expr.with_data([("counter", 30)]).expect_owned(false);
    }

    #[test]
    fn less_than_static() {
        let expr = less_than(unum(2), unum(3));
        expr.test().expect_owned(true);
    }

    #[test]
    fn less_than_equal_static() {
        let expr = less_than_equal(unum(2), unum(3));
        expr.test().expect_owned(true);

        let expr = less_than_equal(unum(3), unum(3));
        expr.test().expect_owned(true);
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
        expr.with_data([("one", 1), ("two", 2)]).eval_bool(true);

        // or
        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", true)])
            .eval_bool(true);

        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", false)])
            .eval_bool(true);

        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", false)])
            .eval_bool(false);

        // and
        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", true)])
            .eval_bool(true);

        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", true)])
            .eval_bool(false);

        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", false)])
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
