use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use anathema_state::{Hex, Number, PendingValue, Subscriber, Type, ValueRef};
use anathema_strings::StrIndex;
use anathema_templates::expressions::{Equality, LogicalOp, Op};
use anathema_templates::{Expression, Primitive};

use crate::value::ValueKind;

macro_rules! or_null {
    ($val:expr) => {
        match $val {
            Some(val) => val,
            None => return ValueExpr::Null,
        }
    };
}

#[derive(Debug)]
pub enum Str<'bp> {
    Borrowed(&'bp str),
    Owned(ValueRef, String),
}

#[derive(Debug, Copy, Clone)]
pub enum Kind<T> {
    Static(T),
    Dyn(PendingValue),
}

impl<'bp> Kind<&'bp str> {
    pub(crate) fn to_str(&self) -> Cow<'bp, str> {
        match self {
            Kind::Static(s) => Cow::Borrowed(s),
            Kind::Dyn(pending_value) => pending_value
                .as_state()
                .map(|s| s.as_str().unwrap().to_owned())
                .unwrap()
                .into(),
        }
    }
}

impl Kind<i64> {
    pub(crate) fn to_int(&self) -> i64 {
        match self {
            Kind::Static(s) => *s,
            Kind::Dyn(pending_value) => pending_value
                .as_state()
                .map(|s| s.as_int().unwrap().to_owned())
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueExpr<'bp> {
    Bool(Kind<bool>),
    Char(Kind<char>),
    Int(Kind<i64>),
    Float(Kind<f64>),
    Hex(Kind<Hex>),
    Str(Kind<&'bp str>),
    DynMap(PendingValue),
    DynList(PendingValue),
    List(Box<[Self]>),
    Map(HashMap<&'bp str, Self>),
    Index(Box<Self>, Box<Self>),

    Not(Box<Self>),
    Negative(Box<Self>),

    Equality(Box<Self>, Box<Self>, Equality),
    LogicalOp(Box<Self>, Box<Self>, LogicalOp),

    Op(Box<Self>, Box<Self>, Op),
    Either(Box<Self>, Box<Self>),

    Call,

    Null,
}

impl<'bp> From<Primitive> for ValueExpr<'bp> {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Bool(b) => Self::Bool(Kind::Static(b)),
            Primitive::Char(c) => Self::Char(Kind::Static(c)),
            Primitive::Int(i) => Self::Int(Kind::Static(i)),
            Primitive::Float(f) => Self::Float(Kind::Static(f)),
            Primitive::Hex(hex) => Self::Hex(Kind::Static(hex)),
        }
    }
}

// Resolve an expression to a value kind, this is the final value in the chain
pub(crate) fn resolve_value<'bp>(expr: &ValueExpr<'bp>, sub: Subscriber) -> ValueKind<'bp> {
    match expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        // -----------------------------------------------------------------------------
        ValueExpr::Bool(Kind::Static(b)) => ValueKind::Bool(*b),
        ValueExpr::Bool(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Bool(state.as_bool().unwrap())
        }
        ValueExpr::Char(Kind::Static(c)) => ValueKind::Char(*c),
        ValueExpr::Char(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Char(state.as_char().unwrap())
        }
        ValueExpr::Int(Kind::Static(i)) => ValueKind::Int(*i),
        ValueExpr::Int(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Int(state.as_int().unwrap())
        }
        ValueExpr::Float(Kind::Static(f)) => ValueKind::Float(*f),
        ValueExpr::Float(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Float(state.as_float().unwrap())
        }
        ValueExpr::Hex(Kind::Static(h)) => ValueKind::Hex(*h),
        ValueExpr::Hex(Kind::Dyn(pending)) => {
            pending.subscribe(sub);
            let state = pending.as_state().unwrap();
            ValueKind::Hex(state.as_hex().unwrap())
        }
        ValueExpr::Str(Kind::Static(s)) => ValueKind::Str(Cow::Borrowed(s)),
        ValueExpr::Str(Kind::Dyn(val)) => {
            let state = val.as_state().unwrap();
            let s = state.as_str().unwrap();
            ValueKind::Str(Cow::Owned(s.to_owned()))
        }

        // -----------------------------------------------------------------------------
        //   - Operations and conditionals -
        // -----------------------------------------------------------------------------
        ValueExpr::Not(value_expr) => {
            let ValueKind::Bool(val) = resolve_value(value_expr, sub) else { return ValueKind::Null };
            ValueKind::Bool(!val)
        }
        ValueExpr::Negative(value_expr) => match resolve_value(value_expr, sub) {
            ValueKind::Int(n) => ValueKind::Int(-n),
            ValueKind::Float(n) => ValueKind::Float(-n),
            _ => ValueKind::Null,
        },
        ValueExpr::Equality(lhs, rhs, equality) => {
            let lhs = resolve_value(lhs, sub);
            let rhs = resolve_value(rhs, sub);
            let b = match equality {
                Equality::Eq => lhs == rhs,
                Equality::NotEq => lhs != rhs,
                Equality::Gt => lhs > rhs,
                Equality::Gte => lhs >= rhs,
                Equality::Lt => lhs < rhs,
                Equality::Lte => lhs <= rhs,
            };
            ValueKind::Bool(b)
        }
        ValueExpr::LogicalOp(lhs, rhs, logical_op) => {
            let ValueKind::Bool(lhs) = resolve_value(lhs, sub) else { return ValueKind::Null };
            let ValueKind::Bool(rhs) = resolve_value(rhs, sub) else { return ValueKind::Null };
            let b = match logical_op {
                LogicalOp::And => lhs && rhs,
                LogicalOp::Or => lhs || rhs,
            };
            ValueKind::Bool(b)
        }
        ValueExpr::Op(lhs, rhs, op) => match (resolve_value(lhs, sub), resolve_value(rhs, sub)) {
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => ValueKind::Int(int_op(lhs, rhs, *op)),
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs as f64, rhs, *op)),
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => ValueKind::Float(float_op(lhs, rhs as f64, *op)),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs, rhs, *op)),
            _ => ValueKind::Null,
        },
        ValueExpr::Either(first, second) => match resolve_value(first, sub) {
            ValueKind::Null => resolve_value(second, sub),
            first => first,
        },

        // -----------------------------------------------------------------------------
        //   - Maps and lists -
        // -----------------------------------------------------------------------------
        ValueExpr::DynMap(value) => {
            let state = value.as_state().unwrap();
            state.as_any_map();
            panic!()
        }
        ValueExpr::DynList(pending_value) => todo!(),
        ValueExpr::List(_) => todo!(),
        ValueExpr::Map(hash_map) => todo!(),
        ValueExpr::Index(src, index) => {
            let expr = resolve_index(src, index, sub);
            resolve_value(&expr, sub)
        }

        // -----------------------------------------------------------------------------
        //   - Call -
        // -----------------------------------------------------------------------------
        ValueExpr::Call => todo!(),

        // -----------------------------------------------------------------------------
        //   - Null -
        // -----------------------------------------------------------------------------
        ValueExpr::Null => ValueKind::Null,
    }
}

fn resolve_pending(val: PendingValue, sub: Subscriber) -> ValueExpr<'static> {
    val.subscribe(sub);
    match val.type_info() {
        Type::Int => ValueExpr::Int(Kind::Dyn(val)),
        Type::Float => ValueExpr::Float(Kind::Dyn(val)),
        Type::Char => ValueExpr::Char(Kind::Dyn(val)),
        Type::String => ValueExpr::Str(Kind::Dyn(val)),
        Type::Bool => ValueExpr::Bool(Kind::Dyn(val)),
        Type::Map => ValueExpr::DynMap(val),
        Type::List => ValueExpr::DynList(val),
        // Type::Composite => ValueKind::Composite,
        val_type => panic!("{val_type:?}"),
    }
}

fn resolve_index<'bp>(src: &ValueExpr<'bp>, index: &ValueExpr<'bp>, sub: Subscriber) -> ValueExpr<'bp> {
    match src {
        ValueExpr::DynMap(value) => {
            let s = or_null!(value.as_state());
            let map = s.as_any_map().expect("a dyn map is always an any_map");
            let key = or_null!(resolve_str(index, sub));
            let val = or_null!(map.lookup(key.to_str()));
            resolve_pending(val, sub)
        }
        ValueExpr::DynList(value) => {
            let s = or_null!(value.as_state());
            let list = s.as_any_list().expect("a dyn list is always an any_list");
            let key = resolve_int(index, sub);
            let val = or_null!(list.lookup(key.to_int() as usize));
            resolve_pending(val, sub)
        }
        ValueExpr::List(_) => todo!(),
        ValueExpr::Map(hash_map) => {
            let key = or_null!(resolve_str(index, sub));
            or_null!(hash_map.get(&*key.to_str()).cloned())
        }
        ValueExpr::Index(inner_src, inner_index) => {
            let src = resolve_index(inner_src, inner_index, sub);
            resolve_index(&src, index, sub)
        }
        ValueExpr::Either(first, second) => {
            let src = match resolve_expr(first, sub) {
                None | Some(ValueExpr::Null) => match resolve_expr(second, sub) {
                    None | Some(ValueExpr::Null) => return ValueExpr::Null,
                    Some(e) => e,
                },
                Some(e) => e,
            };
            resolve_index(&src, index, sub)
        }
        ValueExpr::Null => ValueExpr::Null,
        _ => unreachable!(),
    }
}

fn resolve_expr<'a, 'bp>(expr: &'a ValueExpr<'bp>, sub: Subscriber) -> Option<ValueExpr<'bp>> {
    match expr {
        ValueExpr::Either(first, second) => match resolve_expr(first, sub) {
            None | Some(ValueExpr::Null) => resolve_expr(second, sub),
            expr => expr,
        },
        ValueExpr::Index(src, index) => Some(resolve_index(src, index, sub)),
        _ => None,
        // ValueExpr::Bool(_) |
        // ValueExpr::Char(_) |
        // ValueExpr::Int(_) |
        // ValueExpr::Float(_) |
        // ValueExpr::Hex(_) |
        // ValueExpr::Str(_) => expr,
        // _ => panic!(),
    }
}

fn resolve_str<'a, 'bp>(index: &'a ValueExpr<'bp>, sub: Subscriber) -> Option<Kind<&'bp str>> {
    match index {
        ValueExpr::Str(kind) => Some(*kind),
        ValueExpr::Index(src, index) => match resolve_index(src, index, sub) {
            ValueExpr::Str(kind) => Some(kind),
            _ => None,
        },
        ValueExpr::Either(first, second) => resolve_str(first, sub).or_else(|| resolve_str(second, sub)),
        ValueExpr::Null => None,
        ValueExpr::Call => todo!(),
        _ => None,
    }
}

fn resolve_int<'a, 'bp>(index: &'a ValueExpr<'bp>, sub: Subscriber) -> Kind<i64> {
    match index {
        ValueExpr::Int(kind) => *kind,
        _ => panic!(),
    }
}

fn int_op(lhs: i64, rhs: i64, op: Op) -> i64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    }
}

fn float_op(lhs: f64, rhs: f64, op: Op) -> f64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    }
}

#[cfg(test)]
mod test {
    use anathema_state::Hex;
    use anathema_templates::expressions::{
        and, boolean, chr, either, eq, float, greater_than, greater_than_equal, hex, ident, index, less_than,
        less_than_equal, map, neg, not, num, or, strlit,
    };
    use anathema_state::{AnyValue, List, Map, States};
    use anathema_templates::expressions::{add, div, modulo, mul, sub};
    use anathema_templates::{Globals, Variables};

    use super::*;
    use crate::context::ResolverCtx;
    use crate::immediate::ImmediateResolver;
    use crate::scope::Scope;
    use crate::value::Value;
    use crate::Resolver;

    struct TestCaseBuilder {
        variables: Variables,
        scopes: Scope,
        states: States,
        state: Map<Box<dyn AnyValue>>,
    }

    impl TestCaseBuilder {
        pub fn new(scopes: Scope, states: States) -> Self {
            Self {
                variables: Variables::new(),
                scopes,
                states,
                state: Map::empty(),
            }
        }

        pub fn with_global(mut self, key: &str, value: impl Into<Expression>) -> Self {
            self.variables.declare(key, value);
            self
        }

        pub fn with_state(mut self, key: &str, value: impl AnyValue) -> Self {
            let value: Box<dyn AnyValue> = Box::new(value);
            self.state.insert(key, value);
            self
        }

        pub fn finish(mut self) -> TestCase {
            let globals = self.variables.into();
            let state_id = self.states.insert(self.state);
            self.scopes.insert_state(state_id);
            TestCase {
                globals,
                scopes: self.scopes,
                states: self.states,
            }
        }
    }

    struct TestCase {
        globals: Globals,
        scopes: Scope,
        states: States,
    }

    impl TestCase {
        fn eval<'bp>(&'bp self, expr: &'bp Expression) -> Value<'bp> {
            let ctx = ResolverCtx::new(&self.globals, &self.scopes, &self.states);
            let mut resolver = ImmediateResolver::new(&ctx);
            let value_expr = resolver.resolve(expr);
            let val = Value::new(value_expr, Subscriber::ZERO);
            val
        }
    }

    fn setup() -> TestCaseBuilder {
        let mut scopes = Scope::new();
        let mut states = States::new();
        TestCaseBuilder::new(scopes, states)
    }

    #[test]
    fn either_index() {
        // state[0] ? attributes[0]
        let expr = either(
            index(index(ident("attributes"), strlit("a")), num(0)),
            index(index(ident("state"), strlit("a")), num(0)),
        );

        let mut list = List::empty();
        list.push("a string");

        let test = setup().with_state("a", list).finish();
        let value = test.eval(&*expr);
        assert_eq!("a string", value.to_str().unwrap());
    }

    #[test]
    fn either_then_index() {
        // (state ? attributes)[0]
        let expr = index(
            either(
                index(ident("attributes"), strlit("a")),
                index(ident("state"), strlit("a")),
            ),
            num(0),
        );

        let mut list = List::empty();
        list.push("a string");

        let test = setup().with_state("a", list).finish();
        let value = test.eval(&*expr);
        assert_eq!("a string", value.to_str().unwrap());
    }

    #[test]
    fn either_or() {
        let test = setup().with_state("a", 1).with_state("b", 2).finish();

        // There is no c, so use b
        let expr = either(index(ident("state"), strlit("c")), index(ident("state"), strlit("b")));
        let value = test.eval(&*expr);
        assert_eq!(2, value.to_int().unwrap());

        // There is a, so don't use b
        let expr = either(index(ident("state"), strlit("a")), index(ident("state"), strlit("b")));
        let value = test.eval(&*expr);
        assert_eq!(1, value.to_int().unwrap());
    }

    #[test]
    fn mods() {
        let test = setup().with_state("num", 5).finish();
        let lookup = index(ident("state"), strlit("num"));
        let expr = modulo(lookup, num(3));
        let value = test.eval(&*expr);
        assert_eq!(2, value.to_int().unwrap());
    }

    #[test]
    fn division() {
        let test = setup().with_state("num", 6).finish();
        let lookup = index(ident("state"), strlit("num"));
        let expr = div(lookup, num(2));
        let value = test.eval(&*expr);
        assert_eq!(3, value.to_int().unwrap());
    }

    #[test]
    fn multiplication() {
        let test = setup().with_state("num", 2).finish();
        let lookup = index(ident("state"), strlit("num"));
        let expr = mul(lookup, num(2));
        let value = test.eval(&*expr);
        assert_eq!(4, value.to_int().unwrap());
    }

    #[test]
    fn subtraction() {
        let test = setup().with_state("num", 1).finish();
        let lookup = index(ident("state"), strlit("num"));
        let expr = sub(lookup, num(2));
        let value = test.eval(&*expr);
        assert_eq!(-1, value.to_int().unwrap());
    }

    #[test]
    fn addition() {
        let test = setup().with_state("num", 1).finish();
        let lookup = index(ident("state"), strlit("num"));
        let expr = add(lookup, num(2));
        let value = test.eval(&*expr);
        assert_eq!(3, value.to_int().unwrap());
    }

    #[test]
    fn test_or() {
        let test = setup().finish();
        let is_true = or(boolean(false), boolean(true));
        let is_true = test.eval(&*is_true);
        assert_eq!(true, is_true.to_bool().unwrap());
    }

    #[test]
    fn test_and() {
        let test = setup().finish();
        let is_true = and(boolean(true), boolean(true));
        let is_true = test.eval(&*is_true);
        assert_eq!(true, is_true.to_bool().unwrap());
    }

    #[test]
    fn lte() {
        let test = setup().finish();
        let is_true = less_than_equal(num(1), num(2));
        let is_also_true = less_than_equal(num(1), num(1));
        let is_true = test.eval(&*is_true);
        let is_also_true = test.eval(&*is_also_true);
        assert_eq!(true, is_true.to_bool().unwrap());
        assert_eq!(true, is_also_true.to_bool().unwrap());
    }

    #[test]
    fn lt() {
        let test = setup().finish();
        let is_true = less_than(num(1), num(2));
        let is_false = less_than(num(1), num(1));
        let is_true = test.eval(&*is_true);
        let is_false = test.eval(&*is_false);
        assert_eq!(true, is_true.to_bool().unwrap());
        assert_eq!(false, is_false.to_bool().unwrap());
    }

    #[test]
    fn gte() {
        let test = setup().finish();
        let is_true = greater_than_equal(num(2), num(1));
        let is_also_true = greater_than_equal(num(2), num(2));
        let is_true = test.eval(&*is_true);
        let is_also_true = test.eval(&*is_also_true);
        assert_eq!(true, is_true.to_bool().unwrap());
        assert_eq!(true, is_also_true.to_bool().unwrap());
    }

    #[test]
    fn gt() {
        let test = setup().finish();
        let is_true = greater_than(num(2), num(1));
        let is_false = greater_than(num(2), num(2));
        let is_true = test.eval(&*is_true);
        let is_false = test.eval(&*is_false);
        assert_eq!(true, is_true.to_bool().unwrap());
        assert_eq!(false, is_false.to_bool().unwrap());
    }

    #[test]
    fn equality() {
        let test = setup().finish();
        let is_true = eq(num(1), num(1));
        let is_true = test.eval(&is_true);
        let is_false = &not(eq(num(1), num(1)));
        let is_false = test.eval(is_false);
        assert_eq!(true, is_true.to_bool().unwrap());
        assert_eq!(false, is_false.to_bool().unwrap());
    }

    #[test]
    fn neg_float() {
        let expr = neg(float(123.1));
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(-123.1, value.to_float().unwrap());
    }

    #[test]
    fn neg_num() {
        let expr = neg(num(123));
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(-123, value.to_int().unwrap());
    }

    #[test]
    fn not_true() {
        let expr = not(boolean(false));
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(true, value.to_bool().unwrap());
    }

    #[test]
    fn str_resolve() {
        // state[empty|full]
        let expr = index(ident("state"), either(ident("empty"), ident("full")));
        let test = setup()
            .with_state("key", "a string")
            .with_global("full", "key")
            .finish();
        let value = test.eval(&*expr);
        assert_eq!("a string", value.to_str().unwrap());
    }

    #[test]
    fn state_string() {
        let expr = index(ident("state"), strlit("str"));
        let test = setup().with_state("str", "a string").finish();
        let value = test.eval(&*expr);
        assert_eq!("a string", value.to_str().unwrap());
    }

    #[test]
    fn state_float() {
        let expr = index(ident("state"), strlit("float"));
        let test = setup().with_state("float", 1.2).finish();
        let value = test.eval(&*expr);
        assert_eq!(1.2, value.to_float().unwrap());
    }

    #[test]
    fn test_either() {
        let expr = either(ident("missings"), num(2));
        let test = setup().with_global("missing", 111).finish();
        let value = test.eval(&*expr);
        assert_eq!(2, value.to_int().unwrap());
    }

    #[test]
    fn test_hex() {
        let expr = hex((1, 2, 3));
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(Hex::from((1, 2, 3)), value.to_hex().unwrap());
    }

    #[test]
    fn test_char() {
        let expr = chr('x');
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!('x', value.to_char().unwrap());
    }

    #[test]
    fn test_float() {
        let expr = float(123.123);
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(123.123, value.to_float().unwrap());
    }

    #[test]
    fn test_int() {
        let expr = num(123);
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }

    #[test]
    fn test_bool() {
        let expr = boolean(true);
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert!(value.to_bool().unwrap());
    }

    #[test]
    fn test_dyn_list() {
        let expr = index(index(ident("state"), strlit("list")), num(1));
        let mut list = List::empty();
        list.push(123);
        list.push(456);

        let test = setup().with_state("list", list).finish();
        let value = test.eval(&*expr);
        assert_eq!(456, value.to_int().unwrap());
    }

    #[test]
    fn test_expression_map_state_key() {
        let expr = index(map([("value", 123)]), index(ident("state"), strlit("key")));
        let test = setup().with_state("key", "value").finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }

    #[test]
    fn test_expression_map() {
        let expr = index(map([("value", 123)]), strlit("value"));
        let test = setup().finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }

    #[test]
    fn test_dyn_map_dyn_key() {
        let expr = index(ident("state"), strlit("value"));
        let test = setup().with_state("value", 123).finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }

    #[test]
    fn test_dyn_map() {
        let expr = index(ident("state"), strlit("value"));
        let test = setup().with_state("value", 123).finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }

    #[test]
    fn test_nested_map() {
        let expr = index(index(ident("state"), strlit("blip")), strlit("value"));
        let mut inner_map = Map::empty();
        inner_map.insert("value", 123);

        let test = setup().with_state("blip", inner_map).finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }

    #[test]
    fn test_nested_maps() {
        let expr = index(
            index(index(ident("state"), strlit("value")), strlit("value")),
            strlit("value"),
        );
        let mut inner_map = Map::empty();
        let mut inner_inner_map = Map::empty();
        inner_inner_map.insert("value", 123);
        inner_map.insert("value", inner_inner_map);

        let test = setup().with_state("value", inner_map).finish();
        let value = test.eval(&*expr);
        assert_eq!(123, value.to_int().unwrap());
    }
}
