use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use anathema_state::{Color, Hex, PendingValue, SubTo, Subscriber, Type};
use anathema_store::smallmap::SmallMap;
use anathema_templates::Expression;

use crate::attributes::ValueKey;
use crate::expression::{ValueExpr, ValueThingy, resolve_value};
use crate::immediate::Resolver;
use crate::{AttributeStorage, ResolverCtx};

pub type Values<'bp> = SmallMap<ValueKey<'bp>, Value<'bp>>;

pub fn resolve<'bp>(expr: &'bp Expression, ctx: &ResolverCtx<'_, 'bp>, sub: impl Into<Subscriber>) -> Value<'bp> {
    let resolver = Resolver::new(ctx);
    let value_expr = resolver.resolve(expr);
    Value::new(value_expr, sub.into(), ctx.attribute_storage)
}

pub fn resolve_collection<'bp>(
    expr: &'bp Expression,
    ctx: &ResolverCtx<'_, 'bp>,
    sub: impl Into<Subscriber>,
) -> Collection<'bp> {
    let value = resolve(expr, ctx, sub);
    Collection(value)
}

#[derive(Debug)]
pub struct Collection<'bp>(pub(crate) Value<'bp>);

impl<'bp> Collection<'bp> {
    pub fn reload(&mut self, attributes: &AttributeStorage<'bp>) {
        self.0.reload(attributes)
    }

    pub fn len(&self) -> usize {
        match &self.0.kind {
            ValueKind::List(vec) => vec.len(),
            ValueKind::DynList(value) => {
                let Some(state) = value.as_state() else { return 0 };
                let Some(list) = state.as_any_list() else { return 0 };
                list.len()
            }
            ValueKind::Int(_)
            | ValueKind::Float(_)
            | ValueKind::Bool(_)
            | ValueKind::Char(_)
            | ValueKind::Hex(_)
            | ValueKind::Color(_)
            | ValueKind::Str(_)
            | ValueKind::Composite
            | ValueKind::Attributes
            | ValueKind::Map
            | ValueKind::Null => 0,
        }
    }
}

/// This is the final value for a node attribute / value.
/// This should be evaluated fully for the `ValueKind`
#[derive(Debug)]
pub struct Value<'bp> {
    pub(crate) expr: ValueExpr<'bp>,
    pub(crate) sub: Subscriber,
    pub(crate) kind: ValueKind<'bp>,
    pub(crate) sub_to: SubTo,
}

impl<'bp> Value<'bp> {
    pub fn new(expr: ValueExpr<'bp>, sub: Subscriber, attribute_storage: &AttributeStorage<'bp>) -> Self {
        let mut sub_to = SubTo::Zero;
        let mut ctx = ValueThingy::new(attribute_storage, sub, &mut sub_to);
        let kind = resolve_value(&expr, &mut ctx);
        Self {
            expr,
            sub,
            kind,
            sub_to,
        }
    }

    pub fn reload(&mut self, attribute_storage: &AttributeStorage<'bp>) {
        self.sub_to.unsubscribe(self.sub);
        let mut ctx = ValueThingy::new(attribute_storage, self.sub, &mut self.sub_to);
        self.kind = resolve_value(&self.expr, &mut ctx);
    }

    pub fn try_as<T>(&self) -> Option<T>
    where
        T: for<'a> TryFrom<&'a ValueKind<'a>>,
    {
        (&self.kind).try_into().ok()
    }

    pub fn strings<F>(&self, mut f: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.kind.strings(&mut f);
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        self.sub_to.unsubscribe(self.sub);
    }
}

impl<'a> Deref for Value<'a> {
    type Target = ValueKind<'a>;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl<'a> DerefMut for Value<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

/// This value can never be part of an evaluation chain, only the return value.
/// It should only ever be the final type that is held by a `Value`, at
/// the end of an evaluation
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ValueKind<'bp> {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Hex(Hex),
    Color(Color),
    Str(Cow<'bp, str>),
    Composite,
    Null,

    // NOTE
    // The map is the final value, and is never used as part
    // of an index, for that reason the map doesn't hold any values.
    Map,
    // NOTE
    // The attributes is the final value, and is never used as part
    // of an index, for that reason the attributes doesn't hold any values.
    Attributes,
    List(Box<[ValueKind<'bp>]>),
    DynList(PendingValue),
}

impl ValueKind<'_> {
    pub fn to_int(&self) -> Option<i64> {
        match self.as_int() {
            Some(val) => Some(val),
            None => Some(self.as_float()? as i64),
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        let ValueKind::Int(i) = self else { return None };
        Some(*i)
    }

    pub fn as_float(&self) -> Option<f64> {
        let ValueKind::Float(i) = self else { return None };
        Some(*i)
    }

    pub fn as_bool(&self) -> Option<bool> {
        let ValueKind::Bool(b) = self else { return None };
        Some(*b)
    }

    pub fn as_char(&self) -> Option<char> {
        let ValueKind::Char(i) = self else { return None };
        Some(*i)
    }

    pub fn as_hex(&self) -> Option<Hex> {
        let ValueKind::Hex(i) = self else { return None };
        Some(*i)
    }

    pub fn as_color(&self) -> Option<Color> {
        let ValueKind::Color(i) = self else { return None };
        Some(*i)
    }

    pub fn as_str(&self) -> Option<&str> {
        let ValueKind::Str(i) = &self else { return None };
        Some(&*i)
    }

    pub fn strings<F>(&self, mut f: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.internal_strings(&mut f);
    }

    fn internal_strings<F>(&self, f: &mut F) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        match self {
            ValueKind::Int(n) => f(&n.to_string()),
            ValueKind::Float(n) => f(&n.to_string()),
            ValueKind::Bool(b) => f(&b.to_string()),
            ValueKind::Char(c) => f(&c.to_string()),
            ValueKind::Hex(x) => f(&x.to_string()),
            ValueKind::Color(col) => f(&col.to_string()),
            ValueKind::Str(cow) => f(cow.as_ref()),
            ValueKind::Map => return true,
            ValueKind::List(vec) => vec.iter().take_while(|val| val.internal_strings(f)).count() == vec.len(),
            ValueKind::DynList(value) => dyn_string(*value, f),
            ValueKind::Composite | ValueKind::Attributes => f("<composite>"),
            ValueKind::Null => return true,
        }
    }
}

fn dyn_string<F>(value: PendingValue, f: &mut F) -> bool
where
    F: FnMut(&str) -> bool,
{
    let Some(state) = value.as_state() else { return true };
    let Some(list) = state.as_any_list() else { return true };
    for i in 0..list.len() {
        let value = list.lookup(i).expect("the value exists");
        let Some(state) = value.as_state() else { continue };
        let should_continue = match value.type_info() {
            Type::Int => f(&state.as_int().expect("type info dictates this").to_string()),
            Type::Float => f(&state.as_float().expect("type info dictates this").to_string()),
            Type::Char => f(&state.as_char().expect("type info dictates this").to_string()),
            Type::String => f(&state.as_str().expect("type info dictates this")),
            Type::Bool => f(&state.as_bool().expect("type info dictates this").to_string()),
            Type::Hex => f(&state.as_hex().expect("type info dictates this").to_string()),
            Type::Map => f("<map>"),
            Type::List => dyn_string(value, f),
            Type::Composite => f(&state.as_hex().expect("type info dictates this").to_string()),
            Type::Unit => f(""),
            Type::Color => f(&state.as_color().expect("type info dictates this").to_string()),
        };

        if !should_continue {
            return false;
        }
    }
    true
}

// -----------------------------------------------------------------------------
//   - From impls -
// -----------------------------------------------------------------------------
macro_rules! from_int {
    ($int:ty) => {
        impl From<$int> for ValueKind<'_> {
            fn from(value: $int) -> Self {
                ValueKind::Int(value as i64)
            }
        }
    };
}

from_int!(i64);
from_int!(i32);
from_int!(i16);
from_int!(i8);
from_int!(u64);
from_int!(u32);
from_int!(u16);
from_int!(u8);

impl From<f64> for ValueKind<'_> {
    fn from(value: f64) -> Self {
        ValueKind::Float(value)
    }
}

impl From<f32> for ValueKind<'_> {
    fn from(value: f32) -> Self {
        ValueKind::Float(value as f64)
    }
}

impl From<bool> for ValueKind<'_> {
    fn from(value: bool) -> Self {
        ValueKind::Bool(value)
    }
}

impl From<char> for ValueKind<'_> {
    fn from(value: char) -> Self {
        ValueKind::Char(value)
    }
}

impl From<Hex> for ValueKind<'_> {
    fn from(value: Hex) -> Self {
        ValueKind::Hex(value)
    }
}

impl From<Color> for ValueKind<'_> {
    fn from(value: Color) -> Self {
        ValueKind::Color(value)
    }
}

impl<'bp, T> From<Vec<T>> for ValueKind<'bp>
where
    T: Into<ValueKind<'bp>>,
{
    fn from(value: Vec<T>) -> Self {
        let list = value.into_iter().map(T::into).collect();
        ValueKind::List(list)
    }
}

impl<'a> From<&'a str> for ValueKind<'a> {
    fn from(value: &'a str) -> Self {
        ValueKind::Str(Cow::Borrowed(value))
    }
}

// -----------------------------------------------------------------------------
//   - Try From -
// -----------------------------------------------------------------------------
macro_rules! try_from_valuekind {
    ($t:ty, $kind:ident) => {
        impl TryFrom<&ValueKind<'_>> for $t {
            type Error = ();

            fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
                match value {
                    ValueKind::$kind(val) => Ok(*val),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! try_from_valuekind_int {
    ($t:ty, $kind:ident) => {
        impl TryFrom<&ValueKind<'_>> for $t {
            type Error = ();

            fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
                match value {
                    ValueKind::$kind(val) => Ok(*val as $t),
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_valuekind!(i64, Int);
try_from_valuekind!(f64, Float);
try_from_valuekind!(bool, Bool);
try_from_valuekind!(char, Char);
try_from_valuekind!(Hex, Hex);
try_from_valuekind!(Color, Color);

try_from_valuekind_int!(usize, Int);
try_from_valuekind_int!(i32, Int);
try_from_valuekind_int!(f32, Float);
try_from_valuekind_int!(i16, Int);
try_from_valuekind_int!(i8, Int);
try_from_valuekind_int!(u32, Int);
try_from_valuekind_int!(u16, Int);
try_from_valuekind_int!(u8, Int);

impl<'a, 'bp> TryFrom<&'a ValueKind<'bp>> for &'a str {
    type Error = ();

    fn try_from(value: &'a ValueKind<'bp>) -> Result<Self, Self::Error> {
        match value {
            ValueKind::Str(Cow::Borrowed(val)) => Ok(val),
            ValueKind::Str(Cow::Owned(val)) => Ok(val.as_str()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use anathema_state::{Hex, States};
    use anathema_templates::Variables;
    use anathema_templates::expressions::{
        add, and, boolean, chr, div, either, eq, float, greater_than, greater_than_equal, hex, ident, index, less_than,
        less_than_equal, list, map, modulo, mul, neg, not, num, or, strlit, sub, text_segments,
    };

    use crate::ValueKind;
    use crate::testing::setup;

    #[test]
    fn attribute_lookup() {
        let expr = index(ident("attributes"), strlit("a"));
        let int = num(123);

        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.set_attribute("a", &int);
            let value = test.eval(&*expr);
            assert_eq!(123, value.as_int().unwrap());
        });
    }

    #[test]
    fn expr_list_dyn_index() {
        let expr = index(list([1, 2, 3]), add(ident("index"), num(1)));

        let mut states = States::new();
        let mut globals = Variables::new();
        globals.declare("index", 0);

        setup(&mut states, globals, |test| {
            let value = test.eval(&*expr);
            assert_eq!(2, value.as_int().unwrap());
        });
    }

    #[test]
    fn expr_list() {
        let expr = index(list([1, 2, 3]), num(0));

        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let value = test.eval(&*expr);
            assert_eq!(1, value.as_int().unwrap());
        });
    }

    #[test]
    fn either_index() {
        // state[0] ? attributes[0]
        let expr = either(
            index(index(ident("state"), strlit("list")), num(0)),
            index(index(ident("attributes"), strlit("list")), num(0)),
        );

        let list = list([strlit("from attribute")]);

        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            // Set list for attributes
            test.set_attribute("list", &list);

            // Evaluate the value.
            // The state is not yet set so it will fall back to attributes
            let mut value = test.eval(&*expr);
            assert_eq!("from attribute", value.as_str().unwrap());

            // Set the state value
            test.with_state(|state| state.list.push("from state"));

            // The value now comes from the state
            value.reload(&test.attributes);
            assert_eq!("from state", value.as_str().unwrap());
        });
    }

    #[test]
    fn either_then_index() {
        // (state ? attributes)[0]

        let list = list([num(123)]);
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(
                either(
                    index(ident("attributes"), strlit("list")),
                    index(ident("state"), strlit("list")),
                ),
                num(0),
            );

            test.with_state(|state| state.list.push("a string"));
            let value = test.eval(&*expr);
            assert_eq!("a string", value.as_str().unwrap());

            test.set_attribute("list", &list);
            let value = test.eval(&*expr);
            assert_eq!(123, value.as_int().unwrap());
        });
    }

    #[test]
    fn either_or() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.num.set(1));
            test.with_state(|state| state.num_2.set(2));

            // There is no c, so use b
            let expr = either(
                index(ident("state"), strlit("num_3")),
                index(ident("state"), strlit("num_2")),
            );
            let value = test.eval(&*expr);
            assert_eq!(2, value.as_int().unwrap());

            // There is a, so don't use b
            let expr = either(
                index(ident("state"), strlit("num")),
                index(ident("state"), strlit("num_2")),
            );
            let value = test.eval(&*expr);
            assert_eq!(1, value.as_int().unwrap());
        });
    }

    #[test]
    fn mods() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.num.set(5));
            let lookup = index(ident("state"), strlit("num"));
            let expr = modulo(lookup, num(3));
            let value = test.eval(&*expr);
            assert_eq!(2, value.as_int().unwrap());
        });
    }

    #[test]
    fn division() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.num.set(6));
            let lookup = index(ident("state"), strlit("num"));
            let expr = div(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(3, value.as_int().unwrap());
        });
    }

    #[test]
    fn multiplication() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.num.set(2));
            let lookup = index(ident("state"), strlit("num"));
            let expr = mul(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(4, value.as_int().unwrap());
        });
    }

    #[test]
    fn subtraction() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.num.set(1));
            let lookup = index(ident("state"), strlit("num"));
            let expr = sub(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(-1, value.as_int().unwrap());
        });
    }

    #[test]
    fn addition() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.num.set(1));
            let lookup = index(ident("state"), strlit("num"));
            let expr = add(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(3, value.as_int().unwrap());
        });
    }

    #[test]
    fn test_or() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = or(boolean(false), boolean(true));
            let is_true = test.eval(&*is_true);
            assert_eq!(true, is_true.as_bool().unwrap());
        });
    }

    #[test]
    fn test_and() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = and(boolean(true), boolean(true));
            let is_true = test.eval(&*is_true);
            assert_eq!(true, is_true.as_bool().unwrap());
        });
    }

    #[test]
    fn lte() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = less_than_equal(num(1), num(2));
            let is_also_true = less_than_equal(num(1), num(1));
            let is_true = test.eval(&*is_true);
            let is_also_true = test.eval(&*is_also_true);
            assert_eq!(true, is_true.as_bool().unwrap());
            assert_eq!(true, is_also_true.as_bool().unwrap());
        });
    }

    #[test]
    fn lt() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = less_than(num(1), num(2));
            let is_false = less_than(num(1), num(1));
            let is_true = test.eval(&*is_true);
            let is_false = test.eval(&*is_false);
            assert_eq!(true, is_true.as_bool().unwrap());
            assert_eq!(false, is_false.as_bool().unwrap());
        });
    }

    #[test]
    fn gte() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = greater_than_equal(num(2), num(1));
            let is_also_true = greater_than_equal(num(2), num(2));
            let is_true = test.eval(&*is_true);
            let is_also_true = test.eval(&*is_also_true);
            assert_eq!(true, is_true.as_bool().unwrap());
            assert_eq!(true, is_also_true.as_bool().unwrap());
        });
    }

    #[test]
    fn gt() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = greater_than(num(2), num(1));
            let is_false = greater_than(num(2), num(2));
            let is_true = test.eval(&*is_true);
            let is_false = test.eval(&*is_false);
            assert_eq!(true, is_true.as_bool().unwrap());
            assert_eq!(false, is_false.as_bool().unwrap());
        });
    }

    #[test]
    fn equality() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let is_true = eq(num(1), num(1));
            let is_true = test.eval(&is_true);
            let is_false = &not(eq(num(1), num(1)));
            let is_false = test.eval(is_false);
            assert_eq!(true, is_true.as_bool().unwrap());
            assert_eq!(false, is_false.as_bool().unwrap());
        });
    }

    #[test]
    fn neg_float() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = neg(float(123.1));
            let value = test.eval(&*expr);
            assert_eq!(-123.1, value.as_float().unwrap());
        });
    }

    #[test]
    fn neg_num() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = neg(num(123));
            let value = test.eval(&*expr);
            assert_eq!(-123, value.as_int().unwrap());
        });
    }

    #[test]
    fn not_true() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = not(boolean(false));
            let value = test.eval(&*expr);
            assert_eq!(true, value.as_bool().unwrap());
        });
    }

    #[test]
    fn map_resolve() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = map([("a", 123), ("b", 456)]);
            let value = test.eval(&*expr);
            assert_eq!(ValueKind::Map, value.kind);
        });
    }

    #[test]
    fn str_resolve() {
        // state[empty|full]
        let mut states = States::new();
        let mut globals = Variables::new();
        globals.declare("full", "string");
        setup(&mut states, globals, |test| {
            let expr = index(ident("state"), either(ident("empty"), ident("full")));
            test.with_state(|state| state.string.set("a string"));
            let value = test.eval(&*expr);
            assert_eq!("a string", value.as_str().unwrap());
        });
    }

    #[test]
    fn state_string() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| state.string.set("a string"));
            let expr = index(ident("state"), strlit("string"));
            let value = test.eval(&*expr);
            assert_eq!("a string", value.as_str().unwrap());
        });
    }

    #[test]
    fn state_float() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(ident("state"), strlit("float"));
            test.with_state(|state| state.float.set(1.2));
            let value = test.eval(&*expr);
            assert_eq!(1.2, value.as_float().unwrap());
        });
    }

    #[test]
    fn test_either() {
        let mut states = States::new();
        let mut globals = Variables::new();
        globals.declare("missing", 111);
        setup(&mut states, globals, |test| {
            let expr = either(ident("missings"), num(2));
            let value = test.eval(&*expr);
            assert_eq!(2, value.as_int().unwrap());
        });
    }

    #[test]
    fn test_hex() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = hex((1, 2, 3));
            let value = test.eval(&*expr);
            assert_eq!(Hex::from((1, 2, 3)), value.as_hex().unwrap());
        });
    }

    #[test]
    fn test_char() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = chr('x');
            let value = test.eval(&*expr);
            assert_eq!('x', value.as_char().unwrap());
        });
    }

    #[test]
    fn test_float() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = float(123.123);
            let value = test.eval(&*expr);
            assert_eq!(123.123, value.as_float().unwrap());
        });
    }

    #[test]
    fn test_int() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = num(123);
            let value = test.eval(&*expr);
            assert_eq!(123, value.as_int().unwrap());
        });
    }

    #[test]
    fn test_bool() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = boolean(true);
            let value = test.eval(&*expr);
            assert!(value.as_bool().unwrap());
        });
    }

    #[test]
    fn test_dyn_list() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            test.with_state(|state| {
                state.list.push("abc");
                state.list.push("def");
            });
            let expr = index(index(ident("state"), strlit("list")), num(1));
            let value = test.eval(&*expr);
            assert_eq!("def", value.as_str().unwrap());
        });
    }

    #[test]
    fn test_expression_map_state_key() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(map([("value", 123)]), index(ident("state"), strlit("string")));
            test.with_state(|state| state.string.set("value"));
            let value = test.eval(&*expr);
            assert_eq!(123, value.as_int().unwrap());
        });
    }

    #[test]
    fn test_expression_map() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(map([("value", 123)]), strlit("value"));
            let value = test.eval(&*expr);
            assert_eq!(123, value.as_int().unwrap());
        });
    }

    #[test]
    fn test_state_lookup() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(ident("state"), strlit("num"));
            let value = test.eval(&*expr);
            assert_eq!(0, value.as_int().unwrap());
        });
    }

    #[test]
    fn test_nested_map() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(index(ident("state"), strlit("map")), strlit("value"));
            test.with_state(|state| state.map.to_mut().insert("value", 123));
            let value = test.eval(&*expr);
            assert_eq!(123, value.as_int().unwrap());
        });
    }

    // #[test]
    // fn test_nested_maps() {
    //     let mut states = States::new();
    //     setup(&mut states, Default::default(), |test| {
    //         let expr = index(
    //             index(index(ident("state"), strlit("value")), strlit("value")),
    //             strlit("value"),
    //         );
    //         let mut inner_map = Value::new(Map::empty());
    //         let mut inner_inner_map = Value::new(Map::empty());
    //         inner_inner_map.insert("value", 123);
    //         inner_map.insert("value", inner_inner_map);

    //         test.set_state("value", inner_map);
    //         let value = test.eval(&*expr);
    //         assert_eq!(123, value.as_int().unwrap());
    //     });
    // }

    #[test]
    fn stringify() {
        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = text_segments([strlit("hello"), strlit(" "), strlit("world")]);
            let value = test.eval(&*expr);
            let mut actual = String::new();
            value.strings(|st| {
                actual.push_str(st);
                true
            });
            assert_eq!("hello world", &actual);
        });
    }
}
