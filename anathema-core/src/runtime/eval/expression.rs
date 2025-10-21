use std::collections::HashMap;

use anathema_state::{AnonValue, Color, Hex, Type};
use anathema_store::slab::Key;

use crate::runtime::elements::ElementId;
use crate::runtime::eval::scope::{Entry, ScopeKey};
use crate::runtime::eval::values::TemplateValue;
use crate::runtime::eval::EvalCtx;
use crate::runtime::functions::Function;
use crate::templates::expressions::{Equality, LogicalOp, Op};
use crate::templates::{Expression, Primitive};

#[derive(Debug, Clone)]
pub enum Kind<T> {
    Static(T),
    Dyn(AnonValue, Key),
}

impl<T> Drop for Kind<T> {
    fn drop(&mut self) {
        if let Self::Dyn(val, key) = self {
            val.unsubscribe(*key);
        }
    }
}

#[derive(Debug)]
pub(crate) enum RuntimeExpression<'bp> {
    Bool(Kind<bool>),
    Char(Kind<char>),
    Int(Kind<i64>),
    Float(Kind<f64>),
    Hex(Kind<Hex>),
    Color(Kind<Color>),
    Str(Kind<&'bp str>),
    DynMap(AnonValue, Key),
    DynList(AnonValue, Key),
    Composite(AnonValue, Key),
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
        args: Box<[RuntimeExpression<'bp>]>,
    },

    Null,
}

impl<'bp> From<Primitive> for RuntimeExpression<'bp> {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Bool(val) => RuntimeExpression::Bool(Kind::Static(val)),
            Primitive::Char(val) => RuntimeExpression::Char(Kind::Static(val)),
            Primitive::Int(val) => RuntimeExpression::Int(Kind::Static(val)),
            Primitive::Float(val) => RuntimeExpression::Float(Kind::Static(val)),
            Primitive::Hex(val) => RuntimeExpression::Hex(Kind::Static(val)),
        }
    }
}

impl<'bp> From<(AnonValue, ElementId)> for RuntimeExpression<'bp> {
    fn from((value, element): (AnonValue, ElementId)) -> Self {
        let key = element.into();
        match value.type_info() {
            Type::Int => Self::Int(Kind::Dyn(value, key)),
            Type::Float => Self::Float(Kind::Dyn(value, key)),
            Type::Char => Self::Char(Kind::Dyn(value, key)),
            Type::String => Self::Str(Kind::Dyn(value, key)),
            Type::Bool => Self::Bool(Kind::Dyn(value, key)),
            Type::Hex => Self::Hex(Kind::Dyn(value, key)),
            Type::Color => Self::Color(Kind::Dyn(value, key)),
            Type::Map => Self::DynMap(value, key),
            Type::List => Self::DynList(value, key),
            Type::Composite => Self::Composite(value, key),
            Type::Unit => Self::Null,
            Type::Maybe => todo!(),
        }
    }
}

pub(crate) fn eval_expr<'bp>(
    expr: &'bp Expression,
    element: ElementId,
    ctx: &mut EvalCtx<'_, 'bp>,
) -> RuntimeExpression<'bp> {
    match expr {
        &Expression::Primitive(primitive) => primitive.into(),
        &Expression::Variable(var_id) => match ctx.variables.load(var_id).map(|expr_id| ctx.expressions.get(expr_id)) {
            Some(expr) => eval_expr(expr, element, ctx),
            None => RuntimeExpression::Null,
        },
        Expression::Str(s) => RuntimeExpression::Str(Kind::Static(s)),
        Expression::List(expressions) | Expression::TextSegments(expressions) => {
            let list = expressions.iter().map(|e| eval_expr(e, element, ctx)).collect();
            RuntimeExpression::List(list)
        }
        Expression::Map(map) => RuntimeExpression::Map(
            map.iter()
                .map(|(k, e)| (k.as_str(), eval_expr(e, element, ctx)))
                .collect(),
        ),
        Expression::Not(expression) => RuntimeExpression::Not(Box::new(eval_expr(expression, element, ctx))),
        Expression::Negative(expression) => RuntimeExpression::Negative(Box::new(eval_expr(expression, element, ctx))),
        Expression::Equality(lhs, rhs, eq) => RuntimeExpression::Equality(
            eval_expr(lhs, element, ctx).into(),
            eval_expr(rhs, element, ctx).into(),
            *eq,
        ),
        Expression::LogicalOp(lhs, rhs, op) => RuntimeExpression::LogicalOp(
            eval_expr(lhs, element, ctx).into(),
            eval_expr(rhs, element, ctx).into(),
            *op,
        ),
        Expression::Ident(ident) => lookup(ident, element, ctx),
        Expression::Index(src, index) => RuntimeExpression::Index(
            eval_expr(src, element, ctx).into(),
            eval_expr(index, element, ctx).into(),
        ),
        Expression::Op(lhs, rhs, op) => RuntimeExpression::Op(
            eval_expr(lhs, element, ctx).into(),
            eval_expr(rhs, element, ctx).into(),
            *op,
        ),
        Expression::Either(first, second) => RuntimeExpression::Either(
            eval_expr(first, element, ctx).into(),
            eval_expr(second, element, ctx).into(),
        ),
        Expression::Range(start, end) => RuntimeExpression::Range(
            eval_expr(start, element, ctx).into(),
            eval_expr(end, element, ctx).into(),
        ),
        Expression::Call { fun, args } => {
            match &**fun {
                // function(args)
                Expression::Ident(fun) => match ctx.lookup_function(fun) {
                    Some(fun_ptr) => {
                        let args = args.iter().map(|arg| eval_expr(arg, element, ctx)).collect::<Box<_>>();
                        RuntimeExpression::Call { fun_ptr, args }
                    }
                    None => RuntimeExpression::Null,
                },
                // some.value.function(args)
                Expression::Index(lhs, rhs) => {
                    let first_arg = eval_expr(lhs, element, ctx);
                    let Expression::Str(fun) = &**rhs else { return RuntimeExpression::Null };
                    match ctx.lookup_function(fun) {
                        Some(fun_ptr) => {
                            let args = std::iter::once(first_arg)
                                .chain(args.iter().map(|arg| eval_expr(arg, element, ctx)))
                                .collect::<Box<_>>();
                            RuntimeExpression::Call { fun_ptr, args }
                        }
                        None => RuntimeExpression::Null,
                    }
                }
                _ => RuntimeExpression::Null,
            }
        }
    }
}

// Lookup an ident for an Expression
fn lookup<'bp>(ident: &str, element: ElementId, ctx: &mut EvalCtx<'_, 'bp>) -> RuntimeExpression<'bp> {
    let key = match ident {
        "state" => ScopeKey::State,
        "attributes" => ScopeKey::Attributes,
        key => ScopeKey::Key(key),
    };

    match ctx.scope.lookup(key, element, ctx.elements) {
        Some(Entry::State(state_id)) => {
            let Some(state) = ctx.states.get(state_id) else { return RuntimeExpression::Null };
            let value = state.reference();
            (value, element).into()
        }
        Some(Entry::Attributes(component)) => RuntimeExpression::Attributes(component),
        Some(Entry::Value { value, .. }) => panic! {},
        None => {
            let Some(id) = ctx.variables.global_lookup(ident) else { return RuntimeExpression::Null };
            let expr = ctx.expressions.get(id);
            eval_expr(expr, element, ctx)
        }
    }
}

pub(crate) fn eval_runtime_expr<'bp>(
    expr: &RuntimeExpression<'bp>,
    element: ElementId,
    ctx: &EvalCtx<'_, 'bp>,
) -> TemplateValue<'bp> {
    match expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        // -----------------------------------------------------------------------------
        &RuntimeExpression::Bool(Kind::Static(b)) => TemplateValue::Bool(b),
        RuntimeExpression::Bool(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_bool() {
                Some(b) => TemplateValue::Bool(b),
                None => TemplateValue::Null,
            }
        }
        RuntimeExpression::Char(kind) => todo!(),
        RuntimeExpression::Int(kind) => todo!(),
        RuntimeExpression::Float(kind) => todo!(),
        RuntimeExpression::Hex(kind) => todo!(),
        RuntimeExpression::Color(kind) => todo!(),
        RuntimeExpression::Str(kind) => todo!(),

        // -----------------------------------------------------------------------------
        //   - Maps, lists and range -
        // -----------------------------------------------------------------------------
        RuntimeExpression::DynMap(anon_value, _) => todo!(),
        RuntimeExpression::DynList(anon_value, _) => todo!(),
        RuntimeExpression::Composite(anon_value, _) => todo!(),
        RuntimeExpression::List(runtime_expressions) => todo!(),
        RuntimeExpression::Map(hash_map) => todo!(),
        RuntimeExpression::Range(runtime_expression, runtime_expression1) => todo!(),
        RuntimeExpression::Attributes(key) => todo!(),

        // -----------------------------------------------------------------------------
        //   - Index -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Index(src, index) => resolve_index(src, index, element, ctx),

        // -----------------------------------------------------------------------------
        //   - Ops -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Not(expr) => {
            let value = eval_runtime_expr(expr, element, ctx);
            TemplateValue::Bool(value.truthiness())
        }
        RuntimeExpression::Negative(runtime_expression) => todo!(),
        RuntimeExpression::Equality(runtime_expression, runtime_expression1, equality) => todo!(),
        RuntimeExpression::LogicalOp(runtime_expression, runtime_expression1, logical_op) => todo!(),
        RuntimeExpression::Op(runtime_expression, runtime_expression1, op) => todo!(),
        RuntimeExpression::Either(runtime_expression, runtime_expression1) => todo!(),

        // -----------------------------------------------------------------------------
        //   - Functions -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Call { fun_ptr, args } => todo!(),

        // -----------------------------------------------------------------------------
        //   - Null -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Null => todo!(),
    }
}

macro_rules! or_null {
    ($opt:expr) => {
        match $opt {
            Some(val) => val,
            None => return TemplateValue::Null,
        }
    };
}

enum Either<'bp> {
    Done(TemplateValue<'bp>),
    Continue,
}

impl<'bp> From<TemplateValue<'bp>> for Either<'bp> {
    fn from(e: TemplateValue<'bp>) -> Self {
        Either::Done(e)
    }
}

fn resolve_index<'bp>(
    src: &RuntimeExpression<'bp>,
    index: &RuntimeExpression<'bp>,
    element: ElementId,
    ctx: &EvalCtx<'_, 'bp>,
) -> TemplateValue<'bp> {
    match src {
        RuntimeExpression::DynMap(map, _) | RuntimeExpression::Composite(map, _) => {
            let state = map.as_state();
            let map = or_null!(state.as_any_map());
            let value = match index {
                RuntimeExpression::Str(kind) => match kind {
                    &Kind::Static(s) => map.lookup(s),
                    Kind::Dyn(anon_value, _) => {
                        let s = anon_value.as_state();
                        let s = or_null!(s.as_str());
                        map.lookup(s)
                    }
                },
                _ => return TemplateValue::Null,
            };
            anon_to_template_value(or_null!(value), element)
        }
        RuntimeExpression::DynList(list, _) => {
            let state = list.as_state();
            let list = or_null!(state.as_any_list());
            let value = match index {
                RuntimeExpression::Int(kind) => match kind {
                    &Kind::Static(i) => list.lookup(i as usize),
                    Kind::Dyn(anon_value, _) => {
                        let i = or_null!(anon_value.as_state().as_int());
                        list.lookup(i as usize)
                    }
                },
                _ => return TemplateValue::Null,
            };
            anon_to_template_value(or_null!(value), element)
        }
        RuntimeExpression::List(list) => {
            let value = match index {
                RuntimeExpression::Int(kind) => match kind {
                    &Kind::Static(i) => list.get(i as usize),
                    Kind::Dyn(anon_value, _) => {
                        let i = or_null!(anon_value.as_state().as_int());
                        list.get(i as usize)
                    }
                },
                _ => return TemplateValue::Null,
            };
            match value {
                Some(val) => eval_runtime_expr(val, element, ctx),
                None => TemplateValue::Null,
            }
        }
        RuntimeExpression::Map(map) => todo!(),
        RuntimeExpression::Index(inner_src, inner_index) => {
            panic!()
            // match resolve_index(inner_src, inner_index, element, ctx) {
            //     TemplateValue::Map => todo!(),
            //     TemplateValue::Attributes => todo!(),
            //     TemplateValue::List(template_values) => todo!(),
            //     TemplateValue::DynList(anon_value) => todo!(),
            //     TemplateValue::DynMap(anon_value) => todo!(),
            //     TemplateValue::Composite(anon_value) => todo!(),
            //     TemplateValue::Range(_, _) => todo!(),
            //     _ => TemplateValue::Null
            // }
        }
        RuntimeExpression::Range(from, to) => todo!(),
        RuntimeExpression::Attributes(el) => {
            let attributes = ctx.get_attributes((*el).into());

            match index {
                RuntimeExpression::Str(kind) => match kind {
                    &Kind::Static(s) => attributes.get(s),
                    Kind::Dyn(anon_value, _) => {
                        let s = anon_value.as_state();
                        let s = or_null!(s.as_str());
                        attributes.get(s)
                    }
                },
                _ => TemplateValue::Null,
            }
        }
        RuntimeExpression::Either(first, second) => todo!(),
        RuntimeExpression::Null => TemplateValue::Null,
        _ => unreachable!("should this return null instead?"),
    }
}

fn anon_to_template_value<'a>(value: AnonValue, element: ElementId) -> TemplateValue<'a> {
    match value.type_info() {
        Type::Int => TemplateValue::Int(value.as_state().as_int().expect("type checked")),
        Type::Float => TemplateValue::Float(value.as_state().as_float().expect("type checked")),
        Type::Char => TemplateValue::Char(value.as_state().as_char().expect("type checked")),
        Type::Bool => TemplateValue::Bool(value.as_state().as_bool().expect("type checked")),
        Type::Hex => TemplateValue::Hex(value.as_state().as_hex().expect("type checked")),
        Type::Color => TemplateValue::Color(value.as_state().as_color().expect("type checked")),
        Type::String => TemplateValue::Str(value.as_state().as_str().expect("type checked").to_string().into()),
        Type::Map => TemplateValue::DynMap(value),
        Type::List => TemplateValue::DynList(value),
        Type::Composite => TemplateValue::Composite(value),
        Type::Unit => TemplateValue::Null,
        Type::Maybe => {
            let val = or_null!(value.as_state().as_maybe()).get();
            anon_to_template_value(or_null!(val), element)
        }
    }
}

#[cfg(test)]
mod test {
    use anathema::{List, State, Value};

    use super::*;
    use crate::runtime::eval::testing::with_expr;
    use crate::templates::expressions::{ident, index, num, strlit};
    use crate::testing::{with_blueprint, RunBuilder, TestWidget};

    #[derive(Debug, State)]
    pub struct TestState {
        list: Value<List<u32>>,
        index: Value<usize>,
    }

    #[test]
    fn meh() {
        let expr = index(
            index(
                //
                ident("a"),
                strlit("b"),
            ),
            //
            strlit("c"),
        );

        // Setup some global data
        // {a: {b: {c: 1}}}
        let mut a = HashMap::new();
        let mut b = HashMap::new();
        b.insert("c".to_string(), num(123));
        let b = Expression::from(b);
        a.insert("b".to_string(), b);
        let map = Expression::Map(a);

        let mut test = RunBuilder::new();
        test.register_global("a", map);

        let expr = test.insert_expression(expr);
        let mut inst = test.finish();
        let el = inst.add_widget(TestWidget("hello".to_string()), None);

        inst.run(|ctx| {
            let expr = ctx.expressions.get(expr);
            let rt = eval_expr(expr, el, ctx);
            let x = eval_runtime_expr(&rt, el, ctx);
            panic!("{x:#?}");
        });
    }
}
