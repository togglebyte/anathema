use std::borrow::Cow;
use std::collections::HashMap;

use anathema_state::{Color, Hex, PendingValue, SubTo, Subscriber, Type};
use anathema_store::slab::Key;
use anathema_templates::Primitive;
use anathema_templates::expressions::{Equality, LogicalOp, Op};

use crate::AttributeStorage;
use crate::functions::Function;
use crate::value::ValueKind;

macro_rules! or_null {
    ($val:expr) => {
        match $val {
            Some(val) => val,
            None => return ValueExpr::Null,
        }
    };
}

pub struct ValueResolutionContext<'a, 'bp> {
    pub(crate) sub: Subscriber,
    pub(crate) sub_to: &'a mut SubTo,
    attribute_storage: &'a AttributeStorage<'bp>,
}

impl<'a, 'bp> ValueResolutionContext<'a, 'bp> {
    pub fn new(attribute_storage: &'a AttributeStorage<'bp>, sub: Subscriber, sub_to: &'a mut SubTo) -> Self {
        Self {
            attribute_storage,
            sub,
            sub_to,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Kind<T> {
    Static(T),
    Dyn(PendingValue),
}

#[derive(Debug, Clone)]
pub enum ValueExpr<'bp> {
    Bool(Kind<bool>),
    Char(Kind<char>),
    Int(Kind<i64>),
    Float(Kind<f64>),
    Hex(Kind<Hex>),
    Color(Kind<Color>),
    Str(Kind<&'bp str>),
    DynMap(PendingValue),
    DynList(PendingValue),
    Composite(PendingValue),
    List(Box<[Self]>),
    Map(HashMap<&'bp str, Self>),
    Index(Box<Self>, Box<Self>),
    Attributes(Key),

    Not(Box<Self>),
    Negative(Box<Self>),

    Equality(Box<Self>, Box<Self>, Equality),
    LogicalOp(Box<Self>, Box<Self>, LogicalOp),

    Op(Box<Self>, Box<Self>, Op),
    Either(Box<Self>, Box<Self>),

    Call {
        fun_ptr: &'bp Function,
        args: Box<[ValueExpr<'bp>]>,
    },

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

impl<'bp> From<PendingValue> for ValueExpr<'bp> {
    fn from(value: PendingValue) -> Self {
        match value.type_info() {
            Type::Int => Self::Int(Kind::Dyn(value)),
            Type::Float => Self::Float(Kind::Dyn(value)),
            Type::Char => Self::Char(Kind::Dyn(value)),
            Type::String => Self::Str(Kind::Dyn(value)),
            Type::Bool => Self::Bool(Kind::Dyn(value)),
            Type::Hex => Self::Hex(Kind::Dyn(value)),
            Type::Color => Self::Color(Kind::Dyn(value)),
            Type::Map => Self::DynMap(value),
            Type::List => Self::DynList(value),
            Type::Composite => Self::Composite(value),
            Type::Unit => Self::Null,
            Type::Maybe => todo!(
                "we probably need to add ValueExpr::Maybe(PendingValue) for this, this would be the only place where that variant would make sense"
            ),
        }
    }
}

// Resolve an expression to a value kind, this is the final value in the chain
pub(crate) fn resolve_value<'a, 'bp>(
    value_expr: &ValueExpr<'bp>,
    ctx: &mut ValueResolutionContext<'a, 'bp>,
) -> ValueKind<'bp> {
    match value_expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        // -----------------------------------------------------------------------------
        ValueExpr::Bool(Kind::Static(b)) => ValueKind::Bool(*b),
        ValueExpr::Bool(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_bool() {
                Some(b) => ValueKind::Bool(b),
                None => ValueKind::Null,
            }
        }
        ValueExpr::Char(Kind::Static(c)) => ValueKind::Char(*c),
        ValueExpr::Char(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_char() {
                Some(c) => ValueKind::Char(c),
                None => ValueKind::Null,
            }
        }
        ValueExpr::Int(Kind::Static(i)) => ValueKind::Int(*i),
        ValueExpr::Int(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_int() {
                Some(i) => ValueKind::Int(i),
                None => ValueKind::Null,
            }
        }
        ValueExpr::Float(Kind::Static(f)) => ValueKind::Float(*f),
        ValueExpr::Float(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_float() {
                Some(f) => ValueKind::Float(f),
                None => ValueKind::Null,
            }
        }
        ValueExpr::Hex(Kind::Static(h)) => ValueKind::Hex(*h),
        ValueExpr::Hex(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_hex() {
                Some(h) => ValueKind::Hex(h),
                None => ValueKind::Null,
            }
        }
        ValueExpr::Color(Kind::Static(h)) => ValueKind::Color(*h),
        ValueExpr::Color(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_color() {
                Some(h) => ValueKind::Color(h),
                None => ValueKind::Null,
            }
        }
        ValueExpr::Str(Kind::Static(s)) => ValueKind::Str(Cow::Borrowed(s)),
        ValueExpr::Str(Kind::Dyn(pending)) => {
            pending.subscribe(ctx.sub);
            ctx.sub_to.push(pending.sub_key());
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_str() {
                Some(s) => ValueKind::Str(Cow::Owned(s.to_owned())),
                None => ValueKind::Null,
            }
        }

        // -----------------------------------------------------------------------------
        //   - Operations and conditionals -
        // -----------------------------------------------------------------------------
        ValueExpr::Not(value_expr) => {
            let value = resolve_value(value_expr, ctx);
            ValueKind::Bool(!value.truthiness())
        }
        ValueExpr::Negative(value_expr) => match resolve_value(value_expr, ctx) {
            ValueKind::Int(n) => ValueKind::Int(-n),
            ValueKind::Float(n) => ValueKind::Float(-n),
            _ => ValueKind::Null,
        },
        ValueExpr::Equality(lhs, rhs, equality) => {
            let lhs = resolve_value(lhs, ctx);
            let rhs = resolve_value(rhs, ctx);
            let b = match equality {
                Equality::Eq => lhs.value_eq(&rhs),
                Equality::NotEq => !lhs.value_eq(&rhs),
                Equality::Gt => lhs > rhs,
                Equality::Gte => lhs >= rhs,
                Equality::Lt => lhs < rhs,
                Equality::Lte => lhs <= rhs,
            };
            ValueKind::Bool(b)
        }
        ValueExpr::LogicalOp(lhs, rhs, logical_op) => {
            let ValueKind::Bool(lhs) = resolve_value(lhs, ctx) else { return ValueKind::Null };
            let ValueKind::Bool(rhs) = resolve_value(rhs, ctx) else { return ValueKind::Null };
            let b = match logical_op {
                LogicalOp::And => lhs && rhs,
                LogicalOp::Or => lhs || rhs,
            };
            ValueKind::Bool(b)
        }
        ValueExpr::Op(lhs, rhs, op) => match (resolve_value(lhs, ctx), resolve_value(rhs, ctx)) {
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => ValueKind::Int(int_op(lhs, rhs, *op)),
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs as f64, rhs, *op)),
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => ValueKind::Float(float_op(lhs, rhs as f64, *op)),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs, rhs, *op)),
            _ => ValueKind::Null,
        },
        ValueExpr::Either(first, second) => {
            let value = resolve_value(first, ctx);
            match value {
                ValueKind::Null => resolve_value(second, ctx),
                first => first,
            }
        }

        // -----------------------------------------------------------------------------
        //   - Maps, lists and maybe -
        // -----------------------------------------------------------------------------
        ValueExpr::Map(_) => ValueKind::Map,
        ValueExpr::DynMap(map) => ValueKind::DynMap(*map),
        ValueExpr::Attributes(_) => ValueKind::Attributes,
        ValueExpr::DynList(value) => {
            value.subscribe(ctx.sub);
            ctx.sub_to.push(value.sub_key());
            ValueKind::DynList(*value)
        }
        ValueExpr::List(l) => {
            let values = l.iter().map(|v| resolve_value(v, ctx)).collect();
            ValueKind::List(values)
        }
        ValueExpr::Index(src, index) => {
            let expr = resolve_index(src, index, ctx);
            resolve_value(&expr, ctx)
        }
        ValueExpr::Composite(comp) => ValueKind::Composite(*comp),

        // -----------------------------------------------------------------------------
        //   - Call -
        // -----------------------------------------------------------------------------
        ValueExpr::Call { fun_ptr, args } => {
            let args = args.iter().map(|arg| resolve_value(arg, ctx)).collect::<Box<_>>();
            fun_ptr.invoke(&args)
        }

        // -----------------------------------------------------------------------------
        //   - Null -
        // -----------------------------------------------------------------------------
        ValueExpr::Null => ValueKind::Null,
    }
}

fn resolve_pending<'bp>(val: PendingValue, ctx: &mut ValueResolutionContext<'_, 'bp>) -> ValueExpr<'static> {
    match val.type_info() {
        Type::Int => ValueExpr::Int(Kind::Dyn(val)),
        Type::Float => ValueExpr::Float(Kind::Dyn(val)),
        Type::Char => ValueExpr::Char(Kind::Dyn(val)),
        Type::String => ValueExpr::Str(Kind::Dyn(val)),
        Type::Bool => ValueExpr::Bool(Kind::Dyn(val)),
        Type::Hex => ValueExpr::Hex(Kind::Dyn(val)),
        Type::Color => ValueExpr::Color(Kind::Dyn(val)),
        Type::Map | Type::Composite => ValueExpr::DynMap(val),
        Type::List => ValueExpr::DynList(val),
        Type::Unit => ValueExpr::Null,
        Type::Maybe => {
            let state = or_null!(val.as_state());
            let maybe = or_null!(state.as_maybe());
            // If there is no value, subscribe to the `Maybe`
            let inner = match maybe.get() {
                Some(inner) => inner,
                None => {
                    val.subscribe(ctx.sub);
                    ctx.sub_to.push(val.sub_key());
                    return ValueExpr::Null;
                }
            };
            resolve_pending(inner, ctx)
        }
    }
}

fn resolve_index<'bp>(
    src: &ValueExpr<'bp>,
    index: &ValueExpr<'bp>,
    ctx: &mut ValueResolutionContext<'_, 'bp>,
) -> ValueExpr<'bp> {
    match src {
        ValueExpr::DynMap(value) | ValueExpr::Composite(value) => {
            let state = or_null!(value.as_state());
            let map = match state.as_any_map() {
                Some(map) => map,
                None => {
                    // This will happen in the event of an `Option<DynMap>`
                    // where the `Option` is `None`
                    value.subscribe(ctx.sub);
                    ctx.sub_to.push(value.sub_key());
                    return ValueExpr::Null;
                }
            };

            let key = or_null!(resolve_str(index, ctx));
            let val = map.lookup(&key);

            let val = match val {
                Some(key) => key,
                None => {
                    value.subscribe(ctx.sub);
                    ctx.sub_to.push(value.sub_key());
                    return ValueExpr::Null;
                }
            };

            resolve_pending(val, ctx)
        }
        ValueExpr::DynList(value) => {
            value.subscribe(ctx.sub);
            ctx.sub_to.push(value.sub_key());
            let state = or_null!(value.as_state());
            let list = match state.as_any_list() {
                Some(list) => list,
                None => {
                    value.subscribe(ctx.sub);
                    ctx.sub_to.push(value.sub_key());
                    return ValueExpr::Null;
                }
            };
            let index = resolve_int(index, ctx) as usize;
            let val = list.lookup(index);

            // If the values doesn't exist subscribe to the underlying map / state
            // to get notified when the value does exist.
            //
            // If the value does exist unsubscribe from the underlying map / state
            let val = match val {
                Some(val) => val,
                None => {
                    value.subscribe(ctx.sub);
                    ctx.sub_to.push(value.sub_key());
                    return ValueExpr::Null;
                }
            };
            resolve_pending(val, ctx)
        }
        ValueExpr::Attributes(widget_id) => {
            let key = or_null!(resolve_str(index, ctx));
            let attributes = ctx.attribute_storage.get(*widget_id);
            or_null!(attributes.get_value_expr(&key))
        }
        ValueExpr::List(list) => {
            let index = resolve_int(index, ctx);
            list[index as usize].clone()
        }
        ValueExpr::Map(hash_map) => {
            let key = or_null!(resolve_str(index, ctx));
            or_null!(hash_map.get(&*key).cloned())
        }
        ValueExpr::Index(inner_src, inner_index) => {
            let src = resolve_index(inner_src, inner_index, ctx);
            resolve_index(&src, index, ctx)
        }
        ValueExpr::Either(first, second) => {
            let src = match resolve_expr(first, ctx) {
                None | Some(ValueExpr::Null) => match resolve_expr(second, ctx) {
                    None | Some(ValueExpr::Null) => return ValueExpr::Null,
                    Some(e) => e,
                },
                Some(e) => e,
            };
            resolve_index(&src, index, ctx)
        }
        ValueExpr::Null => ValueExpr::Null,
        // TODO: see unreachable message
        val => unreachable!(
            "resolving index: this should return null eventually: {val:?} (you probably did something like x.y on a string)"
        ),
    }
}

fn resolve_expr<'bp>(expr: &ValueExpr<'bp>, ctx: &mut ValueResolutionContext<'_, 'bp>) -> Option<ValueExpr<'bp>> {
    match expr {
        ValueExpr::Either(first, second) => match resolve_expr(first, ctx) {
            None | Some(ValueExpr::Null) => resolve_expr(second, ctx),
            expr => expr,
        },
        ValueExpr::Index(src, index) => Some(resolve_index(src, index, ctx)),
        _ => None,
    }
}

fn resolve_str<'bp>(index: &ValueExpr<'bp>, ctx: &mut ValueResolutionContext<'_, 'bp>) -> Option<Cow<'bp, str>> {
    match resolve_value(index, ctx) {
        ValueKind::Str(s) => Some(s),
        _ => None,
    }
}

fn resolve_int<'bp>(index: &ValueExpr<'bp>, ctx: &mut ValueResolutionContext<'_, 'bp>) -> i64 {
    let value = resolve_value(index, ctx);
    match value {
        ValueKind::Int(index) => index,
        ValueKind::Bool(false) => 0,
        ValueKind::Bool(true) => 1,
        ValueKind::Float(index) => index as i64,
        ValueKind::Char(_)
        | ValueKind::Hex(_)
        | ValueKind::Color(_)
        | ValueKind::Str(_)
        | ValueKind::Composite(_)
        | ValueKind::Null
        | ValueKind::Map
        | ValueKind::DynMap(_)
        | ValueKind::Attributes
        | ValueKind::List(_)
        | ValueKind::DynList(_) => todo!("resolving int: the value is {value:?}"),
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
    use anathema_state::{Changes, Map, Maybe, States, drain_changes};
    use anathema_templates::expressions::{ident, index, num, strlit};

    use crate::testing::setup;

    #[test]
    fn subscribe_if_not_exist() {
        // In this case the list is empty but it exists

        let mut changes = Changes::empty();
        drain_changes(&mut changes);
        assert!(changes.is_empty());

        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(index(ident("state"), strlit("list")), num(0));

            let mut value = test.eval(&expr);

            assert_eq!(value.as_int(), None);

            test.with_state(|state| state.list.push("a"));

            drain_changes(&mut changes);
            for (subs, _) in changes.drain() {
                for sub in subs.iter() {
                    if sub == value.sub {
                        value.reload(&test.attributes);
                    }
                }
            }

            assert_eq!(value.as_str().unwrap(), "a");
        });
    }

    #[test]
    fn list_preceding_value_removed() {
        let mut changes = Changes::empty();

        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            let expr = index(index(ident("state"), strlit("list")), num(1));

            test.with_state(|state| {
                state.list.push("a");
                state.list.push("b");
                state.list.push("c");
            });

            let mut value = test.eval(&expr);

            assert_eq!(value.as_str().unwrap(), "b");

            test.with_state(|state| state.list.remove(0));

            drain_changes(&mut changes);
            assert!(!changes.is_empty());

            for (subs, _) in changes.drain() {
                if subs.iter().any(|sub| sub == value.sub) {
                    value.reload(&test.attributes);
                }
            }
            assert_eq!(value.as_str().unwrap(), "c");
        });
    }

    #[test]
    fn optional_map_from_empty_to_value() {
        let mut changes = Changes::empty();

        let mut states = States::new();
        setup(&mut states, Default::default(), |test| {
            // let expr = index(index(ident("state"), strlit("opt_map")), strlit("key"));
            let expr = index(ident("state"), strlit("opt_map"));

            let value = test.eval(&expr);
            assert!(value.as_str().is_none());

            test.with_state(|state| {
                let mut map = Map::empty();
                map.insert("key", 123);
                state.opt_map.set(Maybe::some(map));
            });

            drain_changes(&mut changes);
            assert!(!changes.is_empty());
        });
    }
}
