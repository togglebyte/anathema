use std::collections::HashMap;

use anathema_compiler::expressions::{Equality, Expression, ExpressionId, LogicalOp, Op, Primitive};
use anathema_compiler::Hex;
use anathema_store::gen_key;
use anathema_store::remotecell::{RemoteCell, RemoteHandle};
use anathema_store::slab::{Generational, Key, SecondaryMap};

use super::assoc::Associations;
use super::scope::{Entry, ScopeId, ScopeKey};
use super::values::{Collection, TemplateValue};
use super::EvalCtx;
use crate::elements::ElementId;
use crate::functions::Function;
use crate::value::{AnonValue, Type, ValueIndex};

#[derive(Debug)]
struct ExprEntry<'bp> {
    expr: RuntimeExpression<'bp>,
    handle: RemoteHandle<TemplateValue<'bp>>,
    scope_id: Option<ScopeId>,
}

impl<'bp> ExprEntry<'bp> {
    pub fn new(
        scope_id: Option<ScopeId>,
        expr: RuntimeExpression<'bp>,
        handle: RemoteHandle<TemplateValue<'bp>>,
    ) -> Self {
        Self { expr, handle, scope_id }
    }
}

#[derive(Debug)]
struct ExprEntries<'bp> {
    exprs: Vec<ExprEntry<'bp>>,
}

impl<'bp> ExprEntries<'bp> {
    fn new(scope: Option<ScopeId>, expr: RuntimeExpression<'bp>, handle: RemoteHandle<TemplateValue<'bp>>) -> Self {
        Self {
            exprs: vec![ExprEntry::new(scope, expr, handle)],
        }
    }

    fn value(&self, scope_id: Option<ScopeId>) -> Option<RemoteCell<TemplateValue<'bp>>> {
        let entry = self.exprs.iter().find(|entry| entry.scope_id == scope_id)?;
        Some(entry.handle.value())
    }

    fn get_scoped(
        &self,
        scope_id: Option<ScopeId>,
    ) -> Option<(&RemoteHandle<TemplateValue<'bp>>, &RuntimeExpression<'bp>)> {
        let entry = self.exprs.iter().find(|e| e.scope_id == scope_id)?;
        Some((&entry.handle, &entry.expr))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct RTKey {
    expression: ExpressionId,
    scope: Option<ElementId>,
}

#[derive(Debug)]
pub(crate) struct RuntimeExpressions<'bp> {
    // The element id here is how we can scope different values with the same expression id,
    // for instance in a for-loop:
    // ```
    // for x in [1, 2, 3]
    //     text x // <- here x has the same expression id for every iteration
    //            //    even though the value should be different. So by scoping
    //            //    the value with the `Iteration`s ElementId it becomes unique
    //            //    per itration.
    // ```
    inner: SecondaryMap<ExpressionId, ExprEntries<'bp>>,
    associations: Associations,
}

impl<'bp> RuntimeExpressions<'bp> {
    pub fn empty() -> Self {
        Self {
            inner: SecondaryMap::empty(),
            associations: Associations::new(),
        }
    }

    fn get_entry(&self, index: ValueIndex) -> Option<(&RemoteHandle<TemplateValue<'bp>>, &RuntimeExpression<'bp>)> {
        let (expression_id, scope_id) = index.consume();
        let entry = self.inner.get(expression_id)?;
        entry.get_scoped(scope_id)
    }

    fn get_value(&self, index: ValueIndex) -> Option<RemoteCell<TemplateValue<'bp>>> {
        let (expr, key) = index.consume();
        self.inner.get(expr).and_then(|e| e.value(key))
    }

    fn get_expression(&self, index: ValueIndex) -> Option<RuntimeExpression<'bp>> {
        let entry = self.get_entry(index)?.1.clone();
        Some(entry)
    }

    fn insert(
        &mut self,
        id: ExpressionId,
        // Made from the parent elements `ElementId`.
        // There is no guarantee that this element is an actual scope,
        // but it's used to find the nearest scope.
        scope: Option<ScopeId>,
        expr: RuntimeExpression<'bp>,
        value: TemplateValue<'bp>,
    ) -> RemoteCell<TemplateValue<'bp>> {
        let (value, handle) = RemoteCell::new(value);
        let entry = ExprEntries::new(scope, expr, handle);
        self.inner.insert(id, entry);
        value
    }

    // The element will be placed in `dirty_widgets` when the value expression
    // is re-evaluated
    fn associate(&mut self, id: ExpressionId, element: ElementId) {
        self.associations.associate(id, element);
    }
}

#[derive(Debug, Clone)]
enum Kind<T> {
    Static(T),
    Dyn(AnonValue, ValueIndex),
}

impl<T> Drop for Kind<T> {
    fn drop(&mut self) {
        if let Self::Dyn(val, key) = self {
            val.unsubscribe(*key);
        }
    }
}

// It's only runtime expressions that subscribe to changes.
#[derive(Debug, Clone)]
pub(crate) enum RuntimeExpression<'bp> {
    Bool(Kind<bool>),
    Char(Kind<char>),
    Int(Kind<i64>),
    Float(Kind<f64>),
    Hex(Kind<Hex>),
    Color(AnonValue, ValueIndex),
    Str(Kind<&'bp str>),
    DynMap(AnonValue, ValueIndex),
    DynList(AnonValue, ValueIndex),
    Composite(AnonValue, ValueIndex),
    List(Box<[Self]>),
    Map(HashMap<&'bp str, Self>),
    Index(Box<Self>, Box<Self>),
    Range {
        start: Box<Self>,
        end: Box<Self>,
        inclusive: bool,
    },
    Attributes(ElementId),

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

impl RuntimeExpression<'_> {
    fn as_usize(&self) -> Option<usize> {
        match self {
            RuntimeExpression::Int(Kind::Static(int)) => Some(*int as usize),
            RuntimeExpression::Int(Kind::Dyn(int, _)) => int.as_state().as_int().map(|i| i as usize),
            _ => None,
        }
    }

    fn with_str<F, T>(&self, f: F) -> Option<T>
    where
        F: Fn(&str) -> T,
    {
        match self {
            &RuntimeExpression::Str(Kind::Static(s)) => Some(f(s)),
            RuntimeExpression::Str(Kind::Dyn(s, _)) => s.as_state().as_str().map(f),
            _ => None,
        }
    }

    fn from_anon(value: AnonValue, expr: ExpressionId, scope: Option<ScopeId>) -> Self {
        let key = ValueIndex::new(expr, scope);
        match value.type_info() {
            Type::Int => Self::Int(Kind::Dyn(value, key)),
            Type::Float => Self::Float(Kind::Dyn(value, key)),
            Type::Char => Self::Char(Kind::Dyn(value, key)),
            Type::String => Self::Str(Kind::Dyn(value, key)),
            Type::Bool => Self::Bool(Kind::Dyn(value, key)),
            Type::Hex => Self::Hex(Kind::Dyn(value, key)),
            Type::Color => Self::Color(value, key),
            Type::Map => Self::DynMap(value, key),
            Type::List => Self::DynList(value, key),
            Type::Composite => Self::Composite(value, key),
            Type::Unit => Self::Null,
            Type::Maybe => todo!(),
        }
    }
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

pub fn re_evalute_expr<'bp>(idx: ValueIndex, ctx: &EvalCtx<'_, 'bp>) {
    let Some((handle, expr)) = ctx.runtime_expressions.get_entry(idx) else { return };
    let (id, scope) = idx.consume();
    let value = eval_runtime_expr(expr, id, scope, ctx);
    handle.set(value);
}

pub fn eval_collection<'bp>(
    id: ExpressionId,
    element: ElementId,
    parent: Option<ElementId>,
    ctx: &mut EvalCtx<'_, 'bp>,
) -> Collection {
    let (value, index) = eval_by_id(id, element, parent, ctx);
    let len = match &*value {
        TemplateValue::List(list) => list.len() as u32,
        TemplateValue::DynList(list) => list.as_state().as_any_list().expect("type checked").len() as u32,
        &TemplateValue::Range {
            start,
            end,
            inclusive: false,
        } => (end - start) as u32,
        &TemplateValue::Range {
            start,
            end,
            inclusive: true,
        } => (end - start) as u32 + 1,
        _ => 0,
    };
    Collection::new(index, len)
}

pub fn eval_by_id<'bp>(
    id: ExpressionId,
    element: ElementId,
    // the parent is needed for scope lookup
    // as the `element` might not be added to the tree yet at this point.
    parent: Option<ElementId>,
    ctx: &mut EvalCtx<'_, 'bp>,
) -> (RemoteCell<TemplateValue<'bp>>, ValueIndex) {
    let scope = ctx.nearest_scope_id(parent);
    let index = ValueIndex::new(id, scope);

    // If the expression already exist: associate the element with the expression
    // and return a remote cell to the already existing value.
    //
    // This is to ensure that there is only one value per expression
    if let Some(value) = ctx.runtime_expressions.get_value(index) {
        ctx.runtime_expressions.associate(id, element);
        return (value, index);
    }

    let expr = ctx.expressions.get(id);
    let expr = eval_expr(expr, id, scope, ctx);
    let value = eval_runtime_expr(&expr, id, scope, ctx);
    ctx.runtime_expressions.associate(id, element);
    let value = ctx.runtime_expressions.insert(id, scope, expr, value);
    (value, index)
}

fn eval_expr<'bp>(
    expr: &'bp Expression,
    expr_id: ExpressionId,
    scope: Option<ScopeId>,
    ctx: &mut EvalCtx<'_, 'bp>,
) -> RuntimeExpression<'bp> {
    match expr {
        &Expression::Primitive(primitive) => primitive.into(),
        &Expression::Variable(var_id) => ctx
            .variables
            .load(var_id)
            .map(|expr_id| {
                let expr = ctx.expressions.get(expr_id);
                eval_expr(expr, expr_id, scope, ctx)
            })
            .unwrap_or(RuntimeExpression::Null),
        Expression::Str(s) => RuntimeExpression::Str(Kind::Static(s)),
        Expression::List(expressions) | Expression::TextSegments(expressions) => {
            let list = expressions.iter().map(|e| eval_expr(e, expr_id, scope, ctx)).collect();
            RuntimeExpression::List(list)
        }
        Expression::Map(map) => RuntimeExpression::Map(
            map.iter()
                .map(|(k, e)| (k.as_str(), eval_expr(e, expr_id, scope, ctx)))
                .collect(),
        ),
        Expression::Not(expression) => RuntimeExpression::Not(Box::new(eval_expr(expression, expr_id, scope, ctx))),
        Expression::Negative(expression) => {
            RuntimeExpression::Negative(Box::new(eval_expr(expression, expr_id, scope, ctx)))
        }
        Expression::Equality(lhs, rhs, eq) => RuntimeExpression::Equality(
            eval_expr(lhs, expr_id, scope, ctx).into(),
            eval_expr(rhs, expr_id, scope, ctx).into(),
            *eq,
        ),
        Expression::LogicalOp(lhs, rhs, op) => RuntimeExpression::LogicalOp(
            eval_expr(lhs, expr_id, scope, ctx).into(),
            eval_expr(rhs, expr_id, scope, ctx).into(),
            *op,
        ),
        Expression::Ident(ident) => lookup(ident, expr_id, scope, ctx),
        Expression::Index(src, index) => RuntimeExpression::Index(
            eval_expr(src, expr_id, scope, ctx).into(),
            eval_expr(index, expr_id, scope, ctx).into(),
        ),
        Expression::Op(lhs, rhs, op) => RuntimeExpression::Op(
            eval_expr(lhs, expr_id, scope, ctx).into(),
            eval_expr(rhs, expr_id, scope, ctx).into(),
            *op,
        ),
        Expression::Either(first, second) => RuntimeExpression::Either(
            eval_expr(first, expr_id, scope, ctx).into(),
            eval_expr(second, expr_id, scope, ctx).into(),
        ),
        Expression::Range { start, end, inclusive } => RuntimeExpression::Range {
            start: eval_expr(start, expr_id, scope, ctx).into(),
            end: eval_expr(end, expr_id, scope, ctx).into(),
            inclusive: *inclusive,
        },
        Expression::Call { fun, args } => {
            match &**fun {
                // function(args)
                Expression::Ident(fun) => match ctx.lookup_function(fun) {
                    Some(fun_ptr) => {
                        let args = args
                            .iter()
                            .map(|arg| eval_expr(arg, expr_id, scope, ctx))
                            .collect::<Box<_>>();
                        RuntimeExpression::Call { fun_ptr, args }
                    }
                    None => RuntimeExpression::Null,
                },
                // some.value.function(args)
                Expression::Index(lhs, rhs) => {
                    let first_arg = eval_expr(lhs, expr_id, scope, ctx);
                    let Expression::Str(fun) = &**rhs else { return RuntimeExpression::Null };
                    match ctx.lookup_function(fun) {
                        Some(fun_ptr) => {
                            let args = std::iter::once(first_arg)
                                .chain(args.iter().map(|arg| eval_expr(arg, expr_id, scope, ctx)))
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
fn lookup<'bp>(
    ident: &str,
    expr_id: ExpressionId,
    scope: Option<ScopeId>,
    ctx: &mut EvalCtx<'_, 'bp>,
) -> RuntimeExpression<'bp> {
    let key = match ident {
        "state" => ScopeKey::State,
        "attributes" => ScopeKey::Attributes,
        key => ScopeKey::Key(key),
    };

    let Some(scope) = scope else { return RuntimeExpression::Null };

    match ctx.scope.lookup(key, scope, ctx.elements) {
        Some(Entry::State(component_id)) => {
            let Some(state) = ctx.components.get_state(component_id) else { return RuntimeExpression::Null };
            let value = state.reference();
            RuntimeExpression::from_anon(value, expr_id, Some(scope))
        }
        Some(Entry::Attributes(component)) => RuntimeExpression::Attributes(component),
        Some(Entry::Iteration {
            loop_counter,
            collection_key,
            ..
        }) => {
            let expr = ctx.runtime_expressions.get_expression(collection_key);
            let Some(expr) = expr else { return RuntimeExpression::Null };

            let sub = ValueIndex::new(expr_id, Some(scope));
            let index = RuntimeExpression::Int(Kind::Dyn(loop_counter, sub));
            let index = RuntimeExpression::Index(expr.into(), index.into());
            index
        }
        // TODO: this is for the loop counter
        // Some(Entry::Iteration { loop_counter, .. }) => RuntimeExpression::from_anon(loop_counter, expr_id, scope),
        Some(Entry::Value { value, .. }) => panic!("values needs to be scoped"),
        Some(Entry::With { value_key, .. }) => ctx
            .runtime_expressions
            .get_expression(value_key)
            .unwrap_or(RuntimeExpression::Null),
        None => {
            let Some(id) = ctx.variables.global_lookup(ident) else { return RuntimeExpression::Null };
            let expr = ctx.expressions.get(id);
            eval_expr(expr, expr_id, Some(scope), ctx)
        }
    }
}

#[derive(Debug)]
enum LazyExpression<'a, 'bp> {
    Expression(&'a RuntimeExpression<'bp>),
    Value(TemplateValue<'bp>),
}

impl<'bp> From<TemplateValue<'bp>> for LazyExpression<'_, 'bp> {
    fn from(value: TemplateValue<'bp>) -> Self {
        Self::Value(value)
    }
}

macro_rules! or_null {
    ($opt:expr) => {
        match $opt {
            Some(val) => val,
            None => return TemplateValue::Null.into(),
        }
    };
}

fn lazy_eval<'a, 'bp>(
    expr: &'a RuntimeExpression<'bp>,
    expression_id: ExpressionId,
    scope: Option<ScopeId>,
    ctx: &EvalCtx<'_, 'bp>,
) -> LazyExpression<'a, 'bp> {
    match expr {
        &RuntimeExpression::Bool(Kind::Static(val)) => TemplateValue::Bool(val).into(),
        RuntimeExpression::Bool(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_bool() {
                Some(val) => TemplateValue::Bool(val).into(),
                None => TemplateValue::Null.into(),
            }
        }
        &RuntimeExpression::Char(Kind::Static(val)) => TemplateValue::Char(val).into(),
        RuntimeExpression::Char(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_char() {
                Some(val) => TemplateValue::Char(val).into(),
                None => TemplateValue::Null.into(),
            }
        }
        &RuntimeExpression::Int(Kind::Static(value)) => TemplateValue::Int(value).into(),
        RuntimeExpression::Int(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_int() {
                Some(value) => TemplateValue::Int(value).into(),
                None => TemplateValue::Null.into(),
            }
        }
        &RuntimeExpression::Float(Kind::Static(value)) => TemplateValue::Float(value).into(),
        RuntimeExpression::Float(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_float() {
                Some(value) => TemplateValue::Float(value).into(),
                None => TemplateValue::Null.into(),
            }
        }
        &RuntimeExpression::Hex(Kind::Static(value)) => TemplateValue::Hex(value).into(),
        RuntimeExpression::Hex(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_hex() {
                Some(value) => TemplateValue::Hex(value).into(),
                None => TemplateValue::Null.into(),
            }
        }
        &RuntimeExpression::Str(Kind::Static(s)) => TemplateValue::Str((s).into()).into(),
        RuntimeExpression::Str(Kind::Dyn(value, _)) => {
            let state = value.as_state();
            match state.as_str() {
                Some(s) => TemplateValue::Str(s.to_string().into()).into(),
                None => TemplateValue::Null.into(),
            }
        }
        RuntimeExpression::Color(value, _) => {
            let state = value.as_state();
            match state.as_color() {
                Some(c) => TemplateValue::Color(c).into(),
                None => TemplateValue::Null.into(),
            }
        }
        &RuntimeExpression::Range {
            ref start,
            ref end,
            inclusive,
        } => {
            let start = match eval_runtime_expr(start, expression_id, scope, ctx) {
                TemplateValue::Int(num) => num,
                _ => return TemplateValue::Null.into(),
            };
            let end = match eval_runtime_expr(end, expression_id, scope, ctx) {
                TemplateValue::Int(num) => num,
                _ => return TemplateValue::Null.into(),
            };
            TemplateValue::Range { start, end, inclusive }.into()
        }
        RuntimeExpression::DynMap(_, _)
        | RuntimeExpression::DynList(_, _)
        | RuntimeExpression::Composite(_, _)
        | RuntimeExpression::List(_)
        | RuntimeExpression::Map(_) => LazyExpression::Expression(expr),
        RuntimeExpression::Attributes(key) => todo!(),
        RuntimeExpression::Index(src, index) => eval_index(src, index, expression_id, scope, ctx),
        RuntimeExpression::Not(expr) => {
            let value = eval_runtime_expr(expr, expression_id, scope, ctx);
            TemplateValue::Bool(!value.truthiness()).into()
        }
        RuntimeExpression::Negative(num) => match eval_runtime_expr(num, expression_id, scope, ctx) {
            TemplateValue::Int(num) => TemplateValue::Int(-num),
            _ => TemplateValue::Null,
        }
        .into(),
        RuntimeExpression::Equality(lhs, rhs, equality) => {
            let lhs = eval_runtime_expr(lhs, expression_id, scope, ctx);
            if lhs == TemplateValue::Null {
                return TemplateValue::Null.into();
            }

            let rhs = eval_runtime_expr(rhs, expression_id, scope, ctx);
            if rhs == TemplateValue::Null {
                return TemplateValue::Null.into();
            }

            let boolean = match equality {
                Equality::Eq => lhs == rhs,
                Equality::NotEq => lhs != rhs,
                Equality::Gt => or_null!(lhs.as_int()) > or_null!(rhs.as_int()),
                Equality::Gte => or_null!(lhs.as_int()) >= or_null!(rhs.as_int()),
                Equality::Lt => or_null!(lhs.as_int()) < or_null!(rhs.as_int()),
                Equality::Lte => or_null!(lhs.as_int()) <= or_null!(rhs.as_int()),
            };

            TemplateValue::Bool(boolean).into()
        }
        RuntimeExpression::LogicalOp(lhs, rhs, op) => {
            let lhs = or_null!(eval_runtime_expr(lhs, expression_id, scope, ctx).as_bool());
            let rhs = or_null!(eval_runtime_expr(rhs, expression_id, scope, ctx).as_bool());

            let result = match op {
                LogicalOp::And => lhs && rhs,
                LogicalOp::Or => lhs || rhs,
            };

            TemplateValue::Bool(result).into()
        }
        &RuntimeExpression::Op(ref lhs, ref rhs, op) => {
            let lhs = eval_runtime_expr(lhs, expression_id, scope, ctx);
            let rhs = eval_runtime_expr(rhs, expression_id, scope, ctx);

            match (lhs, rhs) {
                (TemplateValue::Int(lhs), TemplateValue::Int(rhs)) => int_op(lhs, rhs, op).into(),
                (TemplateValue::Int(lhs), TemplateValue::Float(rhs)) => float_op(lhs as f64, rhs, op).into(),
                (TemplateValue::Float(lhs), TemplateValue::Int(rhs)) => float_op(lhs, rhs as f64, op).into(),
                (TemplateValue::Float(lhs), TemplateValue::Float(rhs)) => float_op(lhs, rhs, op).into(),
                _ => return TemplateValue::Null.into(),
            }
        }
        RuntimeExpression::Either(first, second) => {
            let first = eval_runtime_expr(first, expression_id, scope, ctx);
            if first.truthiness() {
                return first.into();
            }
            eval_runtime_expr(second, expression_id, scope, ctx).into()
        }
        RuntimeExpression::Call { fun_ptr, args } => todo!(),
        RuntimeExpression::Null => TemplateValue::Null.into(),
        RuntimeExpression::Range { start, end, inclusive } => todo!(),
        RuntimeExpression::Op(runtime_expression, runtime_expression1, op) => todo!(),
    }
}

fn int_op(lhs: i64, rhs: i64, op: Op) -> TemplateValue<'static> {
    let result = match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    };

    TemplateValue::Int(result)
}

fn float_op(lhs: f64, rhs: f64, op: Op) -> TemplateValue<'static> {
    let result = match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    };

    TemplateValue::Float(result)
}

// This is the final value and should be resolved to a template value.
// That means no indices should be resolved at this step, but rather
// by the lazy_eval.
fn eval_runtime_expr<'bp>(
    expr: &RuntimeExpression<'bp>,
    expression_id: ExpressionId,
    scope: Option<ScopeId>,
    ctx: &EvalCtx<'_, 'bp>,
) -> TemplateValue<'bp> {
    let expr = match lazy_eval(expr, expression_id, scope, ctx) {
        LazyExpression::Expression(expr) => expr,
        LazyExpression::Value(template_value) => return template_value,
    };

    match expr {
        RuntimeExpression::Bool(_)
        | RuntimeExpression::Char(_)
        | RuntimeExpression::Int(_)
        | RuntimeExpression::Float(_)
        | RuntimeExpression::Hex(_)
        | RuntimeExpression::Color(_, _)
        | RuntimeExpression::Range { .. }
        | RuntimeExpression::Str(_) => unreachable!("this was evaluated in lazy eval"),
        RuntimeExpression::DynMap(_, _) | RuntimeExpression::DynList(_, _) | RuntimeExpression::Composite(_, _) => {
            unreachable!("this is handled by lazy_eval")
        }
        RuntimeExpression::List(items) => TemplateValue::List(
            items
                .iter()
                .map(|i| eval_runtime_expr(i, expression_id, scope, ctx))
                .collect(),
        ),
        RuntimeExpression::Map(hash_map) => todo!(),
        RuntimeExpression::Attributes(key) => todo!(),
        RuntimeExpression::Index(_, _) => unreachable!("this should be resolved by lazy eval only"),
        RuntimeExpression::Not(expr) => {
            panic!();
        }
        RuntimeExpression::Negative(runtime_expression) => todo!(),
        RuntimeExpression::Equality(runtime_expression, runtime_expression1, equality) => todo!(),
        RuntimeExpression::LogicalOp(runtime_expression, runtime_expression1, logical_op) => todo!(),
        RuntimeExpression::Op(runtime_expression, runtime_expression1, op) => todo!(),
        RuntimeExpression::Either(runtime_expression, runtime_expression1) => todo!(),
        RuntimeExpression::Call { fun_ptr, args } => {
            // NOTE: Should this perhaps be done in the lazy eval instead?
            let args = args
                .iter()
                .map(|expr| eval_runtime_expr(expr, expression_id, scope, ctx))
                .collect::<Box<_>>();
            fun_ptr.invoke(&args)
        }
        RuntimeExpression::Null => TemplateValue::Null,
    }
}

fn eval_index<'a, 'bp>(
    container: &'a RuntimeExpression<'bp>,
    index: &'a RuntimeExpression<'bp>,
    expression_id: ExpressionId,
    scope: Option<ScopeId>,
    ctx: &EvalCtx<'_, 'bp>,
) -> LazyExpression<'a, 'bp> {
    match container {
        RuntimeExpression::DynMap(map, _) | RuntimeExpression::Composite(map, _) => {
            let state = map.as_state();
            let map = or_null!(state.as_any_map());
            let value = or_null!(index.with_str(|key| map.lookup(key)).flatten());
            value.subscribe(ValueIndex::new(expression_id, scope));
            anon_to_template_value(value).into()
        }
        RuntimeExpression::DynList(list, _) => {
            let state = list.as_state();
            let list = or_null!(state.as_any_list());
            let index = or_null!(index.as_usize());
            let value = or_null!(list.lookup(index));
            value.subscribe(ValueIndex::new(expression_id, scope));
            anon_to_template_value(value).into()
        }
        RuntimeExpression::Attributes(el) => {
            let attributes = or_null!(ctx.get_attributes(*el));
            let value = or_null!(index.with_str(|key| attributes.get(key)));
            value.clone().into()
        }
        RuntimeExpression::List(list) => {
            let index = or_null!(index.as_usize());
            let expr = or_null!(list.get(index));
            lazy_eval(expr, expression_id, scope, ctx)
        }
        RuntimeExpression::Map(map) => {
            let expr = or_null!(index.with_str(|key| map.get(key)).flatten());
            lazy_eval(expr, expression_id, scope, ctx)
        }
        RuntimeExpression::Index(inner_container, inner_index) => {
            let container = eval_index(inner_container, inner_index, expression_id, scope, ctx);
            match container {
                LazyExpression::Expression(container) => eval_index(container, index, expression_id, scope, ctx),
                LazyExpression::Value(TemplateValue::DynMap(map)) => {
                    let value = index.with_str(|key| map.as_state().as_any_map().expect("type checked").lookup(key));
                    let value = or_null!(value.flatten());
                    panic!();
                    // value.subscribe(expression_id);
                    anon_to_template_value(value).into()
                }
                LazyExpression::Value(TemplateValue::DynList(list)) => {
                    let index = or_null!(index.as_usize());
                    let value = list.as_state().as_any_list().expect("type checked").lookup(index);
                    let value = or_null!(value);
                    value.subscribe(ValueIndex::new(expression_id, scope));
                    anon_to_template_value(value).into()
                }
                LazyExpression::Value(_) => TemplateValue::Null.into(),
            }
        }
        RuntimeExpression::Range { start, end, inclusive } => {
            let index = or_null!(index.as_usize());
            match lazy_eval(start, expression_id, scope, ctx) {
                LazyExpression::Value(TemplateValue::Int(start)) => TemplateValue::Int(start + index as i64).into(),
                // LazyExpression::Expression(expr) => LazyExpression::Expression(RuntimeExpression::Index(expr, ///
                _ => TemplateValue::Null.into(),
            }
        }
        RuntimeExpression::Either(first, second) => match eval_index(first, index, expression_id, scope, ctx) {
            LazyExpression::Expression(runtime_expression) => todo!(),
            LazyExpression::Value(TemplateValue::Null) => eval_index(second, index, expression_id, scope, ctx),
            LazyExpression::Value(value) => value.into(),
        },
        RuntimeExpression::Null => TemplateValue::Null.into(),
        expr => unreachable!("should this return null instead?: {expr:?}"),
    }
}

fn anon_to_template_value<'a>(value: AnonValue) -> TemplateValue<'a> {
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
            anon_to_template_value(or_null!(val))
        }
    }
}

#[cfg(test)]
mod test {
    use anathema::State;
    use anathema_compiler::expressions;

    use super::*;
    use crate::attributes::Attributes;
    use crate::eval::values;
    use crate::testing::{ExpressionEvaluator, RunBuilder, TestWidget};
    use crate::value::{List, Map, Value};

    #[derive(Debug, State)]
    pub struct TestState {
        list: Value<List<u32>>,
        list_o_lists: Value<List<List<u32>>>,
        map: Value<Map<u32>>,
        map_o_maps: Value<Map<Map<u32>>>,
        index: Value<usize>,
    }

    struct TestComp;

    impl crate::components::Component for TestComp {
        type Message = ();
        type State = TestState;
    }

    fn assert_expr<'a>(expr: impl Into<Expression>, expected: impl Into<TemplateValue<'a>>) {
        let expected = expected.into();
        with_val(expr, |value| assert_eq!(value, &expected));
    }

    fn with_val<F>(expr: impl Into<Expression>, f: F)
    where
        F: Fn(&TemplateValue<'_>),
    {
        let expr = expr.into();
        expr_test(|mut test, state| {
            test.eval(
                expr,
                state,
                |attr| {
                    attr.set("number", 1);
                },
                |val| f(val),
            );
        });
    }

    fn expr_test<F>(f: F)
    where
        F: FnOnce(ExpressionEvaluator, TestState),
    {
        // -----------------------------------------------------------------------------
        //   - State -
        // -----------------------------------------------------------------------------
        let mut state = TestState {
            list: List::from_iter(1..5).into(),
            map: Map::empty().into(),
            map_o_maps: Map::empty().into(),
            list_o_lists: List::from_iter((0..5).map(|i| List::from_iter(i..i + 3))).into(),
            index: 2.into(),
        };

        state.map.to_mut().insert("a", 1);
        state.map.to_mut().insert("b", 2);

        let mut a = Map::empty();
        a.insert("val", 1);
        state.map_o_maps.to_mut().insert("a", a);

        let mut attributes = Attributes::empty();
        attributes.set("number", 123);

        f(ExpressionEvaluator::new(), state)
    }

    #[test]
    fn state_index_lookup() {
        use expressions::{ident, index, strlit};
        assert_expr(index(ident("state"), strlit("index")), values::test::num(2));
    }

    #[test]
    fn global_index_lookup() {
        use expressions::{ident, index, num, strlit};

        let expr = index(
            index(
                //
                ident("map"),
                strlit("b"),
            ),
            //
            strlit("c"),
        );

        expr_test(|mut test, state| {
            let mut b = HashMap::from([("c".to_string(), num(123))]);
            let map = Expression::from(HashMap::from([("b".to_string(), b)]));

            test.register_global("map", map);
            test.eval(*expr, state, |_| {}, |val| assert_eq!(*val, values::test::num(123)));
        });
    }

    #[test]
    fn static_int() {
        let expected = values::test::num(123);
        assert_expr(expressions::num(123), expected);
    }

    #[test]
    fn static_float() {
        let expected = values::test::float(1.23);
        assert_expr(expressions::float(1.23), expected);
    }

    #[test]
    fn static_bool() {
        let expected = values::test::boolean(true);
        assert_expr(expressions::boolean(true), expected);
    }

    #[test]
    fn static_char() {
        let expected = values::test::chr('a');
        assert_expr(expressions::chr('a'), expected);
    }

    #[test]
    fn static_hex() {
        let expected = values::test::hex(Hex::from((1, 2, 3)));
        assert_expr(expressions::hex(Hex::from((1, 2, 3))), expected);
    }

    #[test]
    fn static_str() {
        let expected = values::test::strlit("hello");
        assert_expr(expressions::strlit("hello"), expected);
    }

    #[test]
    fn negative_int() {
        let expected = values::test::num(-1);
        assert_expr(expressions::negative(expressions::num(1)), expected);
    }

    #[test]
    fn equals() {
        use expressions::num;
        let expected = values::test::boolean(true);
        assert_expr(expressions::eq(num(1), num(1)), expected);
        let expected = values::test::boolean(false);
        assert_expr(expressions::eq(num(2), num(1)), expected);
    }

    #[test]
    fn not_equals() {
        use expressions::num;
        let expected = values::test::boolean(true);
        assert_expr(expressions::neq(num(2), num(1)), expected);
    }

    #[test]
    fn gt() {
        use expressions::num;
        let expected = values::test::boolean(true);
        assert_expr(expressions::gt(num(2), num(1)), expected);
        let expected = values::test::boolean(false);
        assert_expr(expressions::gt(num(2), num(2)), expected);
    }

    #[test]
    fn gte() {
        use expressions::num;
        let expected = values::test::boolean(true);
        assert_expr(expressions::gte(num(2), num(1)), expected);
        let expected = values::test::boolean(true);
        assert_expr(expressions::gte(num(2), num(2)), expected);
    }

    #[test]
    fn lt() {
        use expressions::num;
        let expected = values::test::boolean(false);
        assert_expr(expressions::lt(num(2), num(1)), expected);
        let expected = values::test::boolean(false);
        assert_expr(expressions::lt(num(2), num(2)), expected);
    }

    #[test]
    fn lte() {
        use expressions::num;
        let expected = values::test::boolean(false);
        assert_expr(expressions::lte(num(2), num(1)), expected);
        let expected = values::test::boolean(true);
        assert_expr(expressions::lte(num(2), num(2)), expected);
    }

    #[test]
    fn and() {
        use expressions::boolean;
        let expected = values::test::boolean(true);
        assert_expr(expressions::and(boolean(true), boolean(true)), expected);
        let expected = values::test::boolean(false);
        assert_expr(expressions::and(boolean(true), boolean(false)), expected);
    }

    #[test]
    fn or() {
        use expressions::boolean;
        let expected = values::test::boolean(true);
        assert_expr(expressions::or(boolean(true), boolean(true)), expected);
        let expected = values::test::boolean(true);
        assert_expr(expressions::or(boolean(true), boolean(false)), expected);
    }

    #[test]
    fn add() {
        use expressions::num;
        let expected = values::test::num(3);
        assert_expr(expressions::add(num(1), num(2)), expected);
    }

    #[test]
    fn sub() {
        use expressions::num;
        let expected = values::test::num(3);
        assert_expr(expressions::sub(num(5), num(2)), expected);
    }

    #[test]
    fn mul() {
        use expressions::num;
        let expected = values::test::num(10);
        assert_expr(expressions::mul(num(5), num(2)), expected);
    }

    #[test]
    fn div() {
        use expressions::num;
        let expected = values::test::num(10);
        assert_expr(expressions::div(num(100), num(10)), expected);
    }

    #[test]
    fn modulo() {
        use expressions::num;
        let expected = values::test::num(2);
        assert_expr(expressions::modulo(num(8), num(6)), expected);
    }

    #[test]
    fn either() {
        use expressions::{num, strlit};
        use values::test::num as val;
        assert_expr(expressions::either(num(0), num(2)), val(2));
        assert_expr(expressions::either(strlit(""), num(2)), val(2));
    }

    #[test]
    fn complex_either() {
        use expressions::{either, ident, index, list, num, strlit};
        use values::test::num as val;
        let expr = index(
            either(
                // index
                index(ident("state"), strlit("missing")),
                // num
                list([1, 2]),
            ),
            num(0),
        );
        assert_expr(expr, val(1));
    }

    #[test]
    fn range_exclusive() {
        use expressions::num;
        let expected = values::test::range(0, 5, false);
        assert_expr(expressions::range(num(0), num(5), false), expected);
    }

    #[test]
    fn range_inclusive() {
        use expressions::num;
        let expected = values::test::range(0, 5, true);
        assert_expr(expressions::range(num(0), num(5), true), expected);
    }

    #[test]
    fn not_true() {
        use expressions::boolean;
        let expected = values::test::boolean(false);
        assert_expr(expressions::not(boolean(true)), expected);
    }

    #[test]
    fn static_list() {
        let expected = values::test::list([1, 2, 3]);
        assert_expr(expressions::list([1, 2, 3]), expected);
    }

    #[test]
    fn dyn_list() {
        use expressions::{ident, index, strlit};

        with_val(index(ident("state"), strlit("list")), |value| {
            assert!(matches!(value, TemplateValue::DynList(_)));
        });
    }

    #[test]
    fn dyn_map() {
        use expressions::{ident, index, strlit};

        with_val(index(ident("state"), strlit("map")), |value| {
            assert!(matches!(value, TemplateValue::DynMap(_)));
        });
    }

    #[test]
    fn dyn_nested_map() {
        use expressions::{ident, index, strlit};

        let expr = index(
            index(
                index(
                    //
                    ident("state"),
                    strlit("map_o_maps"),
                ),
                strlit("a"),
            ),
            strlit("val"),
        );

        assert_expr(expr, values::test::num(1));
    }

    #[test]
    fn dyn_nested_list() {
        use expressions::{ident, index, num, strlit};

        let expr = index(
            index(
                index(
                    //
                    ident("state"),
                    strlit("list_o_lists"),
                ),
                num(1),
            ),
            num(2),
        );

        assert_expr(expr, values::test::num(3));
    }

    #[test]
    fn attributes() {
        use expressions::{ident, index, num, strlit};
        let expr = index(ident("attributes"), strlit("number"));
        assert_expr(expr, values::test::num(1));
    }
}
