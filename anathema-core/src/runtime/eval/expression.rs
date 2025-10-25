use std::collections::HashMap;

use anathema_state::{AnonValue, Color, Hex, Type};
use anathema_store::key;
use anathema_store::remotecell::{RemoteCell, RemoteHandle};
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::runtime::elements::ElementId;
use crate::runtime::eval::assoc::Associations;
use crate::runtime::eval::scope::{Entry, ScopeKey};
use crate::runtime::eval::values::TemplateValue;
use crate::runtime::eval::EvalCtx;
use crate::runtime::functions::Function;
use crate::templates::expressions::{Equality, LogicalOp, Op};
use crate::templates::{Expression, ExpressionId, Primitive};

#[derive(Debug)]
struct ExprEntry<'bp> {
    expr: RuntimeExpression<'bp>,
    handle: RemoteHandle<TemplateValue<'bp>>,
}

impl<'bp> ExprEntry<'bp> {
    fn new(expr: RuntimeExpression<'bp>, handle: RemoteHandle<TemplateValue<'bp>>) -> Self {
        Self { expr, handle }
    }

    fn value(&self) -> RemoteCell<TemplateValue<'bp>> {
        self.handle.value()
    }
}

#[derive(Debug)]
pub(crate) struct RuntimeExpressions<'bp> {
    inner: SecondaryMap<ExpressionId, ExprEntry<'bp>>,
    associations: Associations,
}

impl<'bp> RuntimeExpressions<'bp> {
    pub fn empty() -> Self {
        Self {
            inner: SecondaryMap::empty(),
            associations: Associations::new(),
        }
    }

    pub fn get(&self, id: ExpressionId) -> Option<RemoteCell<TemplateValue<'bp>>> {
        self.inner.get(id).map(|e| e.handle.value())
    }

    fn insert(&mut self, id: ExpressionId, expr: RuntimeExpression<'bp>, value: TemplateValue<'bp>) -> RemoteCell<TemplateValue<'bp>> {
        let (value, handle) = RemoteCell::new(value);
        let entry = ExprEntry::new(expr, handle);
        self.inner.insert(id, entry);
        value
    }

    fn associate(&mut self, id: ExpressionId, element: ElementId) {
        self.associations.associate(id, element);
    }
}

#[derive(Debug, Clone)]
enum Kind<T> {
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

// It's only runtime expressions that subscribe to changes.
#[derive(Debug)]
pub(crate) enum RuntimeExpression<'bp> {
    Bool(Kind<bool>),
    Char(Kind<char>),
    Int(Kind<i64>),
    Float(Kind<f64>),
    Hex(Kind<Hex>),
    Color(AnonValue, Key),
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
            Type::Color => Self::Color(value, key),
            Type::Map => Self::DynMap(value, key),
            Type::List => Self::DynList(value, key),
            Type::Composite => Self::Composite(value, key),
            Type::Unit => Self::Null,
            Type::Maybe => todo!(),
        }
    }
}

pub fn eval_by_id<'bp>(
    id: ExpressionId,
    element: ElementId,
    ctx: &mut EvalCtx<'_, 'bp>,
) -> RemoteCell<TemplateValue<'bp>> {
    // If the expression already exist: associate the element with the expression
    // and return a remote cell to the already existing value.
    //
    // This is to ensure that there is only one value per expression
    if let Some(value) = ctx.runtime_expressions.get(id) {
        ctx.runtime_expressions.associate(id, element);
        return value;
    }

    let expr = ctx.expressions.get(id);
    let expr = eval_expr(expr, element, ctx);
    let value = eval_runtime_expr(&expr, element, ctx);
    ctx.runtime_expressions.insert(id, expr, value)
}

fn eval_expr<'bp>(expr: &'bp Expression, element: ElementId, ctx: &mut EvalCtx<'_, 'bp>) -> RuntimeExpression<'bp> {
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
        Some(Entry::State(component_id)) => {
            let Some(state) = ctx.components.get_state(component_id) else { return RuntimeExpression::Null };
            let value = state.reference();
            (value, element).into()
        }
        Some(Entry::Attributes(component)) => RuntimeExpression::Attributes(component),
        Some(Entry::Value { value, .. }) => panic!("values needs to be scoped"),
        None => {
            let Some(id) = ctx.variables.global_lookup(ident) else { return RuntimeExpression::Null };
            let expr = ctx.expressions.get(id);
            eval_expr(expr, element, ctx)
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

fn lazy_eval<'a, 'bp>(
    expr: &'a RuntimeExpression<'bp>,
    element: ElementId,
    ctx: &EvalCtx<'_, 'bp>,
) -> LazyExpression<'a, 'bp> {
    match expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        // -----------------------------------------------------------------------------
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

        // -----------------------------------------------------------------------------
        //   - Maps, lists and range -
        // -----------------------------------------------------------------------------
        RuntimeExpression::DynMap(_, _)
        | RuntimeExpression::DynList(_, _)
        | RuntimeExpression::Composite(_, _)
        | RuntimeExpression::List(_)
        | RuntimeExpression::Map(_) => LazyExpression::Expression(expr),
        RuntimeExpression::Range(runtime_expression, runtime_expression1) => todo!(),
        RuntimeExpression::Attributes(key) => todo!(),

        // -----------------------------------------------------------------------------
        //   - Index -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Index(src, index) => eval_index(src, index, element, ctx),

        // -----------------------------------------------------------------------------
        //   - Ops -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Not(expr) => {
            let value = eval_runtime_expr(expr, element, ctx);
            TemplateValue::Bool(value.truthiness()).into()
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
        RuntimeExpression::Null => TemplateValue::Null.into(),
    }
}

// This is the final value and should be resolved to a template value.
// That means no indices should be resolved at this step, but rather
// by the lazy_eval.
fn eval_runtime_expr<'bp>(
    expr: &RuntimeExpression<'bp>,
    element: ElementId,
    ctx: &EvalCtx<'_, 'bp>,
) -> TemplateValue<'bp> {
    let expr = match lazy_eval(expr, element, ctx) {
        LazyExpression::Expression(expr) => expr,
        LazyExpression::Value(template_value) => return template_value,
    };

    match expr {
        // -----------------------------------------------------------------------------
        //   - Primitives -
        //
        //   Primitives are resolved as part of lazy_eval as they are
        //   always TemplateValues
        // -----------------------------------------------------------------------------
        RuntimeExpression::Bool(_)
        | RuntimeExpression::Char(_)
        | RuntimeExpression::Int(_)
        | RuntimeExpression::Float(_)
        | RuntimeExpression::Hex(_)
        | RuntimeExpression::Color(_, _)
        | RuntimeExpression::Str(_) => unreachable!("this was evaluated in lazy eval"),

        // -----------------------------------------------------------------------------
        //   - Maps, lists and range -
        // -----------------------------------------------------------------------------
        RuntimeExpression::DynMap(anon_value, _) => todo!(),
        RuntimeExpression::DynList(anon_value, _) => todo!(),
        RuntimeExpression::Composite(anon_value, _) => todo!(),
        RuntimeExpression::List(items) => {
            TemplateValue::List(items.iter().map(|i| eval_runtime_expr(i, element, ctx)).collect())
        }
        RuntimeExpression::Map(hash_map) => todo!(),
        RuntimeExpression::Range(runtime_expression, runtime_expression1) => todo!(),
        RuntimeExpression::Attributes(key) => todo!(),

        // -----------------------------------------------------------------------------
        //   - Index -
        // -----------------------------------------------------------------------------
        RuntimeExpression::Index(_, _) => unreachable!("this should be resolved by lazy eval only"), // resolve_index(src, index, element, ctx),

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
            None => return TemplateValue::Null.into(),
        }
    };
}

fn eval_index<'a, 'bp>(
    container: &'a RuntimeExpression<'bp>,
    index: &'a RuntimeExpression<'bp>,
    element: ElementId,
    ctx: &EvalCtx<'_, 'bp>,
) -> LazyExpression<'a, 'bp> {
    match container {
        RuntimeExpression::DynMap(map, _) | RuntimeExpression::Composite(map, _) => {
            let state = map.as_state();
            let map = or_null!(state.as_any_map());
            let value = or_null!(index.with_str(|key| map.lookup(key)).flatten());
            anon_to_template_value(value, element).into()
        }
        RuntimeExpression::DynList(list, _) => {
            let state = list.as_state();
            let list = or_null!(state.as_any_list());
            let index = or_null!(index.as_usize());
            let value = or_null!(list.lookup(index));
            anon_to_template_value(value, element).into()
        }
        RuntimeExpression::List(list) => {
            let index = or_null!(index.as_usize());
            let expr = or_null!(list.get(index));
            lazy_eval(expr, element, ctx)
        }
        RuntimeExpression::Map(map) => {
            let expr = or_null!(index.with_str(|key| map.get(key)).flatten());
            lazy_eval(expr, element, ctx)
        }
        RuntimeExpression::Index(inner_container, inner_index) => {
            let container = eval_index(inner_container, inner_index, element, ctx);
            match container {
                LazyExpression::Expression(container) => eval_index(container, index, element, ctx),
                LazyExpression::Value(TemplateValue::DynMap(map)) => {
                    let value = index.with_str(|key| map.as_state().as_any_map().expect("type checked").lookup(key));
                    anon_to_template_value(or_null!(value.flatten()), element).into()
                }
                LazyExpression::Value(TemplateValue::DynList(list)) => {
                    let index = or_null!(index.as_usize());
                    let value = list.as_state().as_any_list().expect("type checked").lookup(index);
                    anon_to_template_value(or_null!(value), element).into()
                }
                LazyExpression::Value(_) => TemplateValue::Null.into(),
            }
        }
        RuntimeExpression::Range(from, to) => todo!(),
        RuntimeExpression::Attributes(el) => {
            panic!()
            // let attributes = ctx.get_attributes((*el).into());

            // match index {
            //     RuntimeExpression::Str(kind) => match kind {
            //         &Kind::Static(s) => attributes.get(s),
            //         Kind::Dyn(anon_value, _) => {
            //             let s = anon_value.as_state();
            //             let s = or_null!(s.as_str());
            //             attributes.get(s)
            //         }
            //     },
            //     _ => TemplateValue::Null,
            // }
        }
        RuntimeExpression::Either(first, second) => todo!(),
        RuntimeExpression::Null => TemplateValue::Null.into(),
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
    use anathema::{List, Map, State, Value};

    use super::*;
    use crate::runtime::eval::values;
    use crate::templates::expressions;
    use crate::testing::{RunBuilder, TestWidget};

    #[derive(Debug, State)]
    pub struct TestState {
        list: Value<List<u32>>,
        list_o_lists: Value<List<List<u32>>>,
        map: Value<Map<u32>>,
        map_o_maps: Value<Map<Map<u32>>>,
        index: Value<usize>,
    }

    struct TestComp;

    impl crate::runtime::components::Component for TestComp {
        type Message = ();
        type State = TestState;
    }

    fn assert_expr<'a>(expr: impl Into<Expression>, expected: impl Into<TemplateValue<'a>>) {
        let expected = expected.into();
        with_val(expr, |value| assert_eq!(value, expected));
    }

    fn with_val<F>(expr: impl Into<Expression>, f: F)
    where
        F: Fn(TemplateValue<'_>),
    {
        use expressions::{ident, index, num, strlit};
        let expr = expr.into();

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

        // let expr = index(ident("state"), strlit("index"));

        // -----------------------------------------------------------------------------
        //   - Setup globals -
        //  {map: {b: {c: 1}}}
        //
        //  Setup a global var named map
        // -----------------------------------------------------------------------------
        let mut b = HashMap::from([("c".to_string(), num(123))]);
        let map = Expression::from(HashMap::from([("b".to_string(), b)]));

        let mut test = RunBuilder::new();
        test.register_global("map", map);

        let expr = test.insert_expression(expr);

        // -----------------------------------------------------------------------------
        //   - Setup component and state -
        // -----------------------------------------------------------------------------
        let comp_id = test.add_component(TestComp, state);
        let mut inst = test.finish();
        let comp_el = inst.add_component(comp_id, None);

        // -----------------------------------------------------------------------------
        //   - Add a widget -
        // -----------------------------------------------------------------------------
        let el = inst.add_widget(TestWidget("hello".to_string()), Some(comp_el));
        // ... and scope the state
        inst.scope.push_component(comp_el, comp_id);

        inst.run(|ctx| {
            let expr = ctx.expressions.get(expr);
            let rt = eval_expr(expr, el, ctx);
            let val = eval_runtime_expr(&rt, el, ctx);
            f(val);
        });
    }

    #[test]
    fn state_index_lookup() {
        use expressions::{ident, index, strlit};
        assert_expr(index(ident("state"), strlit("index")), values::num(2));
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
        assert_expr(expr, values::num(123));
    }

    #[test]
    fn static_int() {
        let expected = values::num(123);
        assert_expr(expressions::num(123), expected);
    }

    #[test]
    fn static_float() {
        let expected = values::float(1.23);
        assert_expr(expressions::float(1.23), expected);
    }

    #[test]
    fn static_bool() {
        let expected = values::boolean(true);
        assert_expr(expressions::boolean(true), expected);
    }

    #[test]
    fn static_char() {
        let expected = values::chr('a');
        assert_expr(expressions::chr('a'), expected);
    }

    #[test]
    fn static_hex() {
        let expected = values::hex(Hex::from((1, 2, 3)));
        assert_expr(expressions::hex(Hex::from((1, 2, 3))), expected);
    }

    #[test]
    fn static_str() {
        let expected = values::strlit("hello");
        assert_expr(expressions::strlit("hello"), expected);
    }

    #[test]
    fn static_list() {
        let expected = values::list([1, 2, 3]);
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

        assert_expr(expr, values::num(1));
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

        assert_expr(expr, values::num(3));
    }
}
