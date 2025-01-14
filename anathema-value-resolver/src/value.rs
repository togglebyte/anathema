use std::borrow::Cow;

use anathema_state::{Hex, PendingValue, Subscriber};

use crate::expression::{resolve_value, ValueExpr};

/// This is the final value for a node attribute / value.
/// This should be evaluated fully for the `ValueKind`
#[derive(Debug)]
pub struct Value<'bp> {
    expr: ValueExpr<'bp>,
    pub(crate) sub: Subscriber,
    pub(crate) kind: ValueKind<'bp>,
}

impl<'bp> Value<'bp> {
    pub fn new(expr: ValueExpr<'bp>, sub: Subscriber) -> Self {
        let kind = resolve_value(&expr, sub);
        Self { expr, sub, kind }
    }

    pub fn reload(&mut self) {
        self.expr.unsubscribe(self.sub);
        self.kind = resolve_value(&self.expr, self.sub);
    }

    pub fn to_int(&self) -> Option<i64> {
        let ValueKind::Int(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_float(&self) -> Option<f64> {
        let ValueKind::Float(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_bool(&self) -> Option<bool> {
        let ValueKind::Bool(b) = self.kind else { return None };
        Some(b)
    }

    pub fn to_char(&self) -> Option<char> {
        let ValueKind::Char(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_hex(&self) -> Option<Hex> {
        let ValueKind::Hex(i) = self.kind else { return None };
        Some(i)
    }

    pub fn to_str(&self) -> Option<&str> {
        let ValueKind::Str(i) = &self.kind else { return None };
        Some(i)
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        eprintln!("unsubscribe the value");
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
    Str(Cow<'bp, str>),
    // DynMap(PendingValue),
    // DynList(PendingValue),
    Composite,
    Null,

    // NOTE: It's not possible to get values out of the map / list without
    //       having the correct scope, so this should only ever happen with a proper
    //       resolution context.
    Map,
    List(Vec<ValueKind<'bp>>),
    DynList(PendingValue),
}

#[cfg(test)]
pub(crate) mod test {
    use anathema_state::{AnyState, Hex, List, Map, StateId, States};
    use anathema_templates::expressions::{
        add, and, boolean, chr, div, either, eq, float, greater_than, greater_than_equal, hex, ident, index, less_than,
        less_than_equal, map, modulo, mul, neg, not, num, or, strlit, sub,
    };

    use crate::testing::setup;

    #[test]
    fn either_index() {
        // state[0] ? attributes[0]
        let expr = either(
            index(index(ident("attributes"), strlit("a")), num(0)),
            index(index(ident("state"), strlit("a")), num(0)),
        );

        let mut list = List::empty();
        list.push("a string");

        setup().finish(|mut test| {
            test.set_state("a", list);
            let value = test.eval(&*expr);
            assert_eq!("a string", value.to_str().unwrap());
        });
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

        setup().finish(|mut test| {
            test.set_state("a", list);
            let value = test.eval(&*expr);
            assert_eq!("a string", value.to_str().unwrap());
        });
    }

    #[test]
    fn either_or() {
        setup().finish(|mut test| {
            test.set_state("a", 1);
            test.set_state("b", 2);

            // There is no c, so use b
            let expr = either(index(ident("state"), strlit("c")), index(ident("state"), strlit("b")));
            let value = test.eval(&*expr);
            assert_eq!(2, value.to_int().unwrap());

            // There is a, so don't use b
            let expr = either(index(ident("state"), strlit("a")), index(ident("state"), strlit("b")));
            let value = test.eval(&*expr);
            assert_eq!(1, value.to_int().unwrap());
        });
    }

    #[test]
    fn mods() {
        setup().finish(|mut test| {
            test.set_state("num", 5);
            let lookup = index(ident("state"), strlit("num"));
            let expr = modulo(lookup, num(3));
            let value = test.eval(&*expr);
            assert_eq!(2, value.to_int().unwrap());
        });
    }

    #[test]
    fn division() {
        setup().finish(|mut test| {
            test.set_state("num", 6);
            let lookup = index(ident("state"), strlit("num"));
            let expr = div(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(3, value.to_int().unwrap());
        });
    }

    #[test]
    fn multiplication() {
        setup().finish(|mut test| {
            test.set_state("num", 2);
            let lookup = index(ident("state"), strlit("num"));
            let expr = mul(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(4, value.to_int().unwrap());
        });
    }

    #[test]
    fn subtraction() {
        setup().finish(|mut test| {
            test.set_state("num", 1);
            let lookup = index(ident("state"), strlit("num"));
            let expr = sub(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(-1, value.to_int().unwrap());
        });
    }

    #[test]
    fn addition() {
        setup().finish(|mut test| {
            test.set_state("num", 1);
            let lookup = index(ident("state"), strlit("num"));
            let expr = add(lookup, num(2));
            let value = test.eval(&*expr);
            assert_eq!(3, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_or() {
        setup().finish(|test| {
            let is_true = or(boolean(false), boolean(true));
            let is_true = test.eval(&*is_true);
            assert_eq!(true, is_true.to_bool().unwrap());
        });
    }

    #[test]
    fn test_and() {
        setup().finish(|test| {
            let is_true = and(boolean(true), boolean(true));
            let is_true = test.eval(&*is_true);
            assert_eq!(true, is_true.to_bool().unwrap());
        });
    }

    #[test]
    fn lte() {
        setup().finish(|test| {
            let is_true = less_than_equal(num(1), num(2));
            let is_also_true = less_than_equal(num(1), num(1));
            let is_true = test.eval(&*is_true);
            let is_also_true = test.eval(&*is_also_true);
            assert_eq!(true, is_true.to_bool().unwrap());
            assert_eq!(true, is_also_true.to_bool().unwrap());
        });
    }

    #[test]
    fn lt() {
        setup().finish(|test| {
            let is_true = less_than(num(1), num(2));
            let is_false = less_than(num(1), num(1));
            let is_true = test.eval(&*is_true);
            let is_false = test.eval(&*is_false);
            assert_eq!(true, is_true.to_bool().unwrap());
            assert_eq!(false, is_false.to_bool().unwrap());
        });
    }

    #[test]
    fn gte() {
        setup().finish(|test| {
            let is_true = greater_than_equal(num(2), num(1));
            let is_also_true = greater_than_equal(num(2), num(2));
            let is_true = test.eval(&*is_true);
            let is_also_true = test.eval(&*is_also_true);
            assert_eq!(true, is_true.to_bool().unwrap());
            assert_eq!(true, is_also_true.to_bool().unwrap());
        });
    }

    #[test]
    fn gt() {
        setup().finish(|test| {
            let is_true = greater_than(num(2), num(1));
            let is_false = greater_than(num(2), num(2));
            let is_true = test.eval(&*is_true);
            let is_false = test.eval(&*is_false);
            assert_eq!(true, is_true.to_bool().unwrap());
            assert_eq!(false, is_false.to_bool().unwrap());
        });
    }

    #[test]
    fn equality() {
        setup().finish(|test| {
            let is_true = eq(num(1), num(1));
            let is_true = test.eval(&is_true);
            let is_false = &not(eq(num(1), num(1)));
            let is_false = test.eval(is_false);
            assert_eq!(true, is_true.to_bool().unwrap());
            assert_eq!(false, is_false.to_bool().unwrap());
        });
    }

    #[test]
    fn neg_float() {
        setup().finish(|test| {
            let expr = neg(float(123.1));
            let value = test.eval(&*expr);
            assert_eq!(-123.1, value.to_float().unwrap());
        });
    }

    #[test]
    fn neg_num() {
        setup().finish(|test| {
            let expr = neg(num(123));
            let value = test.eval(&*expr);
            assert_eq!(-123, value.to_int().unwrap());
        });
    }

    #[test]
    fn not_true() {
        let test = setup().finish(|test| {
            let expr = not(boolean(false));
            let value = test.eval(&*expr);
            assert_eq!(true, value.to_bool().unwrap());
        });
    }

    #[test]
    fn str_resolve() {
        // state[empty|full]
        setup().with_global("full", "key").finish(|mut test| {
            let expr = index(ident("state"), either(ident("empty"), ident("full")));
            test.set_state("key", "a string");
            let value = test.eval(&*expr);
            assert_eq!("a string", value.to_str().unwrap());
        });
    }

    #[test]
    fn state_string() {
        setup().finish(|mut test| {
            test.set_state("str", "a string");
            let expr = index(ident("state"), strlit("str"));
            let value = test.eval(&*expr);
            assert_eq!("a string", value.to_str().unwrap());
        });
    }

    #[test]
    fn state_float() {
        setup().finish(|mut test| {
            let expr = index(ident("state"), strlit("float"));
            test.set_state("float", 1.2);
            let value = test.eval(&*expr);
            assert_eq!(1.2, value.to_float().unwrap());
        });
    }

    #[test]
    fn test_either() {
        setup().with_global("missing", 111).finish(|test| {
            let expr = either(ident("missings"), num(2));
            let value = test.eval(&*expr);
            assert_eq!(2, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_hex() {
        let test = setup().finish(|test| {
            let expr = hex((1, 2, 3));
            let value = test.eval(&*expr);
            assert_eq!(Hex::from((1, 2, 3)), value.to_hex().unwrap());
        });
    }

    #[test]
    fn test_char() {
        setup().finish(|test| {
            let expr = chr('x');
            let value = test.eval(&*expr);
            assert_eq!('x', value.to_char().unwrap());
        });
    }

    #[test]
    fn test_float() {
        setup().finish(|test| {
            let expr = float(123.123);
            let value = test.eval(&*expr);
            assert_eq!(123.123, value.to_float().unwrap());
        });
    }

    #[test]
    fn test_int() {
        setup().finish(|test| {
            let expr = num(123);
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_bool() {
        setup().finish(|test| {
            let expr = boolean(true);
            let value = test.eval(&*expr);
            assert!(value.to_bool().unwrap());
        });
    }

    #[test]
    fn test_dyn_list() {
        setup().finish(|mut test| {
            let mut list = List::empty();
            list.push(123);
            list.push(456);
            test.set_state("list", list);

            let expr = index(index(ident("state"), strlit("list")), num(1));
            let value = test.eval(&*expr);
            assert_eq!(456, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_expression_map_state_key() {
        setup().finish(|mut test| {
            let expr = index(map([("value", 123)]), index(ident("state"), strlit("key")));
            test.set_state("key", "value");
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_expression_map() {
        setup().finish(|test| {
            let expr = index(map([("value", 123)]), strlit("value"));
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_dyn_map_dyn_key() {
        setup().finish(|mut test| {
            let expr = index(ident("state"), strlit("value"));
            test.set_state("value", 123);
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_dyn_map() {
        setup().finish(|mut test| {
            let expr = index(ident("state"), strlit("value"));
            test.set_state("value", 123);
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_nested_map() {
        setup().finish(|mut test| {
            let expr = index(index(ident("state"), strlit("blip")), strlit("value"));
            let mut inner_map = Map::empty();
            inner_map.insert("value", 123);

            test.set_state("blip", inner_map);
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }

    #[test]
    fn test_nested_maps() {
        setup().finish(|mut test| {
            let expr = index(
                index(index(ident("state"), strlit("value")), strlit("value")),
                strlit("value"),
            );
            let mut inner_map = Map::empty();
            let mut inner_inner_map = Map::empty();
            inner_inner_map.insert("value", 123);
            inner_map.insert("value", inner_inner_map);

            test.set_state("value", inner_map);
            let value = test.eval(&*expr);
            assert_eq!(123, value.to_int().unwrap());
        });
    }
}
