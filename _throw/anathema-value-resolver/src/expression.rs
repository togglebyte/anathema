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
            None => return ResolvedExpr::Null,
        }
    };
}

#[derive(Debug, Default, Copy, Clone)]
pub(crate) enum ResolvedState {
    Resolved,
    PartiallyResolved,
    #[default]
    Unresolved,
}

pub struct ValueResolutionContext<'a, 'bp> {
    pub(crate) sub: Subscriber,
    attribute_storage: &'a AttributeStorage<'bp>,
    pub(crate) resolved_state: ResolvedState,
    pub(crate) sub_keys: SubTo,
}

impl<'a, 'bp> ValueResolutionContext<'a, 'bp> {
    pub fn new(attribute_storage: &'a AttributeStorage<'bp>, sub: Subscriber, resolved_state: ResolvedState) -> Self {
        Self {
            attribute_storage,
            sub,
            resolved_state,
            sub_keys: SubTo::empty(),
        }
    }

    pub(crate) fn force_sub(&mut self, pending: &PendingValue) {
        pending.subscribe(self.sub);
        self.sub_keys.push(pending.sub_key());
    }

    fn maybe_subscribe(&mut self, pending: &PendingValue) {
        match self.resolved_state {
            ResolvedState::Resolved => (),
            ResolvedState::PartiallyResolved => (),
            ResolvedState::Unresolved => {
                pending.subscribe(self.sub);
                self.sub_keys.push(pending.sub_key());
            }
        }
    }

    fn resolved(&mut self, pending: &PendingValue) {
        match self.resolved_state {
            ResolvedState::Resolved => (),
            ResolvedState::PartiallyResolved | ResolvedState::Unresolved => pending.subscribe(self.sub),
        }

        self.resolved_state = ResolvedState::Resolved;
    }

    fn partially_resolved(&mut self, pending: &PendingValue) {
        self.resolved_state = ResolvedState::PartiallyResolved;
        self.sub_keys.push(pending.sub_key());
        pending.subscribe(self.sub);
    }

    fn is_partially_resolved(&self) -> bool {
        match self.resolved_state {
            ResolvedState::PartiallyResolved => true,
            ResolvedState::Resolved | ResolvedState::Unresolved => false,
        }
    }

    pub(crate) fn done(&mut self) {
        if let ResolvedState::Unresolved = self.resolved_state {
            self.resolved_state = ResolvedState::Resolved;
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Kind<T> {
    Static(T),
    Dyn(PendingValue),
}

// -----------------------------------------------------------------------------
//   - Resolved expression -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum ResolvedExpr<'bp> {
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
    Range(Box<Self>, Box<Self>),
    Attributes(Key),

    Not(Box<Self>),
    Negative(Box<Self>),

    Equality(Box<Self>, Box<Self>, Equality),
    LogicalOp(Box<Self>, Box<Self>, LogicalOp),

    Op(Box<Self>, Box<Self>, Op),
    Either(Box<Self>, Box<Self>),

    Call {
        fun_ptr: &'bp Function,
        args: Box<[ResolvedExpr<'bp>]>,
    },

    Null,
}

impl<'bp> From<Primitive> for ResolvedExpr<'bp> {
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

impl<'bp> From<PendingValue> for ResolvedExpr<'bp> {
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
    value_expr: &ResolvedExpr<'bp>,
    ctx: &mut ValueResolutionContext<'a, 'bp>,
) -> ValueKind<'bp> {
    match value_expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        // -----------------------------------------------------------------------------
        ResolvedExpr::Bool(Kind::Static(b)) => ValueKind::Bool(*b),
        ResolvedExpr::Bool(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_bool() {
                Some(b) => ValueKind::Bool(b),
                None => ValueKind::Null,
            }
        }
        ResolvedExpr::Char(Kind::Static(c)) => ValueKind::Char(*c),
        ResolvedExpr::Char(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_char() {
                Some(c) => ValueKind::Char(c),
                None => ValueKind::Null,
            }
        }
        ResolvedExpr::Int(Kind::Static(i)) => ValueKind::Int(*i),
        ResolvedExpr::Int(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_int() {
                Some(i) => ValueKind::Int(i),
                None => ValueKind::Null,
            }
        }
        ResolvedExpr::Float(Kind::Static(f)) => ValueKind::Float(*f),
        ResolvedExpr::Float(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_float() {
                Some(f) => ValueKind::Float(f),
                None => ValueKind::Null,
            }
        }
        ResolvedExpr::Hex(Kind::Static(h)) => ValueKind::Hex(*h),
        ResolvedExpr::Hex(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_hex() {
                Some(h) => ValueKind::Hex(h),
                None => ValueKind::Null,
            }
        }
        ResolvedExpr::Color(Kind::Static(h)) => ValueKind::Color(*h),
        ResolvedExpr::Color(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_color() {
                Some(h) => ValueKind::Color(h),
                None => ValueKind::Null,
            }
        }
        ResolvedExpr::Str(Kind::Static(s)) => ValueKind::Str(Cow::Borrowed(s)),
        ResolvedExpr::Str(Kind::Dyn(pending)) => {
            ctx.maybe_subscribe(pending);
            let Some(state) = pending.as_state() else { return ValueKind::Null };
            match state.as_str() {
                Some(s) => ValueKind::Str(Cow::Owned(s.to_owned())),
                None => ValueKind::Null,
            }
        }

        // -----------------------------------------------------------------------------
        //   - Operations and conditionals -
        // -----------------------------------------------------------------------------
        ResolvedExpr::Not(value_expr) => {
            let value = resolve_value(value_expr, ctx);
            ValueKind::Bool(!value.truthiness())
        }
        ResolvedExpr::Negative(value_expr) => match resolve_value(value_expr, ctx) {
            ValueKind::Int(n) => ValueKind::Int(-n),
            ValueKind::Float(n) => ValueKind::Float(-n),
            _ => ValueKind::Null,
        },
        ResolvedExpr::Equality(lhs, rhs, equality) => {
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
        ResolvedExpr::LogicalOp(lhs, rhs, logical_op) => {
            let ValueKind::Bool(lhs) = resolve_value(lhs, ctx) else { return ValueKind::Null };
            let ValueKind::Bool(rhs) = resolve_value(rhs, ctx) else { return ValueKind::Null };
            let b = match logical_op {
                LogicalOp::And => lhs && rhs,
                LogicalOp::Or => lhs || rhs,
            };
            ValueKind::Bool(b)
        }
        ResolvedExpr::Op(lhs, rhs, op) => match (resolve_value(lhs, ctx), resolve_value(rhs, ctx)) {
            (ValueKind::Int(lhs), ValueKind::Int(rhs)) => ValueKind::Int(int_op(lhs, rhs, *op)),
            (ValueKind::Int(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs as f64, rhs, *op)),
            (ValueKind::Float(lhs), ValueKind::Int(rhs)) => ValueKind::Float(float_op(lhs, rhs as f64, *op)),
            (ValueKind::Float(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs, rhs, *op)),
            _ => ValueKind::Null,
        },
        ResolvedExpr::Either(first, second) => {
            let value = resolve_value(first, ctx);
            if value.truthiness() {
                return value;
            }
            resolve_value(second, ctx)
        }
        ResolvedExpr::Range(from, to) => {
            let from = match resolve_int(from, ctx) {
                Some(i) => i,
                None => return ValueKind::Null,
            };

            let to = match resolve_int(to, ctx) {
                Some(i) => i,
                None => return ValueKind::Null,
            };
            ValueKind::Range(from, to)
        }

        // -----------------------------------------------------------------------------
        //   - Maps, lists and maybe -
        // -----------------------------------------------------------------------------
        ResolvedExpr::Map(_) => ValueKind::Map,
        ResolvedExpr::DynMap(map) => ValueKind::DynMap(*map),
        ResolvedExpr::Attributes(_) => ValueKind::Attributes,
        ResolvedExpr::DynList(value) => {
            ctx.maybe_subscribe(value);
            ValueKind::DynList(*value)
        }
        ResolvedExpr::List(l) => {
            let values = l.iter().map(|v| resolve_value(v, ctx)).collect();
            ValueKind::List(values)
        }
        ResolvedExpr::Index(src, index) => {
            let expr = resolve_index(src, index, ctx);
            resolve_value(&expr, ctx)
        }
        ResolvedExpr::Composite(comp) => ValueKind::Composite(*comp),

        // -----------------------------------------------------------------------------
        //   - Call -
        // -----------------------------------------------------------------------------
        ResolvedExpr::Call { fun_ptr, args } => {
            let args = args.iter().map(|arg| resolve_value(arg, ctx)).collect::<Box<_>>();
            fun_ptr.invoke(&args)
        }

        // -----------------------------------------------------------------------------
        //   - Null -
        // -----------------------------------------------------------------------------
        ResolvedExpr::Null => ValueKind::Null,
    }
}

fn resolve_pending<'bp>(val: PendingValue, ctx: &mut ValueResolutionContext<'_, 'bp>) -> ResolvedExpr<'static> {
    match val.type_info() {
        Type::Int => ResolvedExpr::Int(Kind::Dyn(val)),
        Type::Float => ResolvedExpr::Float(Kind::Dyn(val)),
        Type::Char => ResolvedExpr::Char(Kind::Dyn(val)),
        Type::String => ResolvedExpr::Str(Kind::Dyn(val)),
        Type::Bool => ResolvedExpr::Bool(Kind::Dyn(val)),
        Type::Hex => ResolvedExpr::Hex(Kind::Dyn(val)),
        Type::Color => ResolvedExpr::Color(Kind::Dyn(val)),
        Type::Map | Type::Composite => ResolvedExpr::DynMap(val),
        Type::List => ResolvedExpr::DynList(val),
        Type::Unit => ResolvedExpr::Null,
        Type::Maybe => {
            let state = or_null!(val.as_state());
            let maybe = or_null!(state.as_maybe());
            // If there is no value, subscribe to the `Maybe`
            let inner = match maybe.get() {
                Some(inner) => {
                    ctx.maybe_subscribe(&val);
                    inner
                }
                None => {
                    ctx.maybe_subscribe(&val);
                    return ResolvedExpr::Null;
                }
            };
            resolve_pending(inner, ctx)
        }
    }
}

fn resolve_index<'bp>(
    src: &ResolvedExpr<'bp>,
    index: &ResolvedExpr<'bp>,
    ctx: &mut ValueResolutionContext<'_, 'bp>,
) -> ResolvedExpr<'bp> {
    match src {
        ResolvedExpr::DynMap(value) | ResolvedExpr::Composite(value) => {
            let state = or_null!(value.as_state());
            let map = match state.as_any_map() {
                Some(map) => map,
                None => return ResolvedExpr::Null,
            };

            let key = or_null!(resolve_str(index, ctx));
            let val = map.lookup(&key);

            let val = match val {
                Some(val) => {
                    if ctx.is_partially_resolved() {
                        value.unsubscribe(ctx.sub);
                    }
                    val
                }
                None => {
                    ctx.partially_resolved(value);
                    return ResolvedExpr::Null;
                }
            };

            ctx.resolved(&val);
            resolve_pending(val, ctx)
        }
        ResolvedExpr::DynList(value) => {
            let state = or_null!(value.as_state());
            let list = match state.as_any_list() {
                Some(list) => list,
                None => return ResolvedExpr::Null,
            };

            let index = or_null!(resolve_int(index, ctx));
            let val = list.lookup(index);

            // If the values doesn't exist subscribe to the underlying map / state
            // to get notified when the value does exist.
            //
            // If the value does exist unsubscribe from the underlying map / state
            let val = match val {
                Some(val) => {
                    if ctx.is_partially_resolved() {
                        value.unsubscribe(ctx.sub);
                    }
                    val
                }
                None => {
                    ctx.partially_resolved(value);
                    return ResolvedExpr::Null;
                }
            };
            resolve_pending(val, ctx)
        }
        ResolvedExpr::Attributes(widget_id) => {
            let key = or_null!(resolve_str(index, ctx));
            let attributes = ctx.attribute_storage.get(*widget_id);
            let value = attributes.get_value_expr(&key);
            or_null!(value)
        }
        ResolvedExpr::List(list) => {
            let index = or_null!(resolve_int(index, ctx));
            match list.get(index).cloned() {
                Some(val) => val,
                None => ResolvedExpr::Null,
            }
        }
        ResolvedExpr::Map(hash_map) => {
            let key = or_null!(resolve_str(index, ctx));
            or_null!(hash_map.get(&*key).cloned())
        }
        ResolvedExpr::Index(inner_src, inner_index) => {
            let src = resolve_index(inner_src, inner_index, ctx);
            resolve_index(&src, index, ctx)
        }
        ResolvedExpr::Either(first, second) => {
            let src = match resolve_expr(first, ctx) {
                None | Some(ResolvedExpr::Null) => match resolve_expr(second, ctx) {
                    None | Some(ResolvedExpr::Null) => return ResolvedExpr::Null,
                    Some(e) => e,
                },
                Some(e) => e,
            };
            resolve_index(&src, index, ctx)
        }
        ResolvedExpr::Null => ResolvedExpr::Null,
        // TODO: see unreachable message
        val => unreachable!(
            "resolving index: this should return null eventually: {val:?} (you probably did something like x.y on a string)"
        ),
    }
}

pub(crate) fn resolve_expr<'bp>(
    expr: &ResolvedExpr<'bp>,
    ctx: &mut ValueResolutionContext<'_, 'bp>,
) -> Option<ResolvedExpr<'bp>> {
    match expr {
        ResolvedExpr::Either(first, second) => match resolve_expr(first, ctx) {
            None | Some(ResolvedExpr::Null) => resolve_expr(second, ctx),
            expr => expr,
        },
        ResolvedExpr::Index(src, index) => Some(resolve_index(src, index, ctx)),
        _ => None,
    }
}

fn resolve_str<'bp>(index: &ResolvedExpr<'bp>, ctx: &mut ValueResolutionContext<'_, 'bp>) -> Option<Cow<'bp, str>> {
    match resolve_value(index, ctx) {
        ValueKind::Str(s) => Some(s),
        _ => None,
    }
}

fn resolve_int<'bp>(index: &ResolvedExpr<'bp>, ctx: &mut ValueResolutionContext<'_, 'bp>) -> Option<usize> {
    let value = resolve_value(index, ctx);
    match value {
        ValueKind::Int(index) => Some(index as usize),
        ValueKind::Bool(false) => Some(0),
        ValueKind::Bool(true) => Some(1),
        ValueKind::Float(index) => Some(index as usize),
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
        | ValueKind::Range(..)
        | ValueKind::DynList(_) => None,
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
    // use anathema_state::{drain_changes, Changes, Map, Maybe, States};
    // use anathema_templates::expressions::{ident, index, num, strlit};

    // use crate::testing::setup;

    // #[test]
    // fn subscribe_if_not_exist() {
    //     // In this case the list is empty but it exists

    //     let mut changes = Changes::empty();
    //     drain_changes(&mut changes);
    //     assert!(changes.is_empty());

    //     let mut states = States::new();
    //     setup(&mut states, Default::default(), |test| {
    //         let expr = index(index(ident("state"), strlit("list")), num(0));

    //         let mut value = test.eval(*expr);

    //         assert_eq!(value.as_int(), None);

    //         test.with_state(|state| state.list.push("a"));

    //         drain_changes(&mut changes);
    //         for (subs, _) in changes.drain() {
    //             for sub in subs.iter() {
    //                 if sub == value.sub {
    //                     value.reload(&test.attributes);
    //                 }
    //             }
    //         }

    //         assert_eq!(value.as_str().unwrap(), "a");
    //     });
    // }

    // #[test]
    // fn list_preceding_value_removed() {
    //     let mut changes = Changes::empty();
    //     let mut states = States::new();

    //     setup(&mut states, Default::default(), |test| {
    //         // state.list[1]
    //         let expr = index(index(ident("state"), strlit("list")), num(1));

    //         test.with_state(|state| {
    //             state.list.push("a");
    //             state.list.push("b");
    //             state.list.push("c");
    //         });

    //         let mut value = test.eval(*expr);

    //         assert_eq!(value.as_str().unwrap(), "b");

    //         test.with_state(|state| state.list.remove(0));

    //         drain_changes(&mut changes);
    //         assert!(!changes.is_empty());

    //         for (subs, _) in changes.drain() {
    //             if subs.iter().any(|sub| sub == value.sub) {
    //                 value.reload(&test.attributes);
    //             }
    //         }
    //         assert_eq!(value.as_str().unwrap(), "c");
    //     });
    // }

    // #[test]
    // fn optional_map_from_empty_to_value() {
    //     let mut changes = Changes::empty();

    //     let mut states = States::new();
    //     setup(&mut states, Default::default(), |test| {
    //         // let expr = index(index(ident("state"), strlit("opt_map")), strlit("key"));
    //         let expr = index(ident("state"), strlit("opt_map"));

    //         let value = test.eval(*expr);
    //         assert!(value.as_str().is_none());

    //         test.with_state(|state| {
    //             let mut map = Map::empty();
    //             map.insert("key", 123);
    //             state.opt_map.set(Maybe::some(map));
    //         });

    //         drain_changes(&mut changes);
    //         assert!(!changes.is_empty());
    //     });
    // }
}
