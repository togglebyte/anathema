use std::collections::HashMap;
use std::ops::ControlFlow;
use std::rc::Rc;

use anathema_state::{register_future, CommonVal, Number, Path, PendingValue, SharedState, States, ValueRef};
use anathema_templates::expressions::{Equality, Op};
use anathema_templates::{Expression, Globals};

use crate::scope::{Scope, ScopeLookup};
use crate::values::{Collection, ValueId};
use crate::Value;

pub(crate) fn future_value<'a>(id: ValueId) -> EvalValue<'a> {
    register_future(id);
    EvalValue::Empty
}

pub enum Either<'a> {
    Static(CommonVal<'a>),
    Dyn(SharedState<'a>),
}

impl<'a> Either<'a> {
    pub fn load_bool(&self) -> bool {
        match self {
            Either::Static(val) => val.to_bool(),
            Either::Dyn(state) => state.to_common().map(|v| v.to_bool()).unwrap_or(false),
        }
    }

    pub fn load_number(&self) -> Option<Number> {
        match self {
            Either::Static(val) => val.to_number(),
            Either::Dyn(state) => state.to_common().and_then(|v| v.to_number()),
        }
    }

    pub(crate) fn load_path(&self) -> Option<Path<'_>> {
        match self {
            Either::Static(CommonVal::Int(n)) => Some(Path::Index(*n as usize)),
            Either::Static(CommonVal::Str(s)) => Some(Path::Key(s)),
            Either::Static(_) => None,
            Either::Dyn(state) => match state.to_common()? {
                CommonVal::Int(n) => Some(Path::Index(n as usize)),
                CommonVal::Str(s) => Some(Path::Key(s)),
                _ => None,
            },
        }
    }

    pub fn to_common(&'a self) -> Option<CommonVal<'a>> {
        match self {
            Either::Static(val) => Some(*val),
            Either::Dyn(state) => state.to_common(),
        }
    }
}

impl<'a> From<CommonVal<'a>> for Either<'a> {
    fn from(value: CommonVal<'a>) -> Self {
        Self::Static(value)
    }
}

impl<'a> From<Number> for Either<'a> {
    fn from(value: Number) -> Self {
        Self::Static(CommonVal::from(value))
    }
}

#[derive(Debug)]
pub(crate) struct Downgraded<'bp>(EvalValue<'bp>);

impl<'bp> Downgraded<'bp> {
    pub(crate) fn upgrade(&self, value_id: ValueId) -> EvalValue<'bp> {
        self.0.inner_upgrade(value_id)
    }
}

#[derive(Debug, PartialEq)]
pub enum EvalValue<'bp> {
    Static(CommonVal<'bp>),
    Dyn(ValueRef),
    Index(Box<Self>, Box<Self>),
    /// Pending value is used for collections
    /// and traversing state, as a means
    /// to access state without subscribing to it
    Pending(PendingValue),
    ExprMap(HashMap<Rc<str>, EvalValue<'bp>>),
    ExprList(Box<[EvalValue<'bp>]>),

    // Operations
    Negative(Box<Self>),
    Op(Box<Self>, Box<Self>, Op),

    // Equality
    Not(Box<Self>),
    Equality(Box<Self>, Box<Self>, Equality),

    Empty,
}

impl<'bp> EvalValue<'bp> {
    fn copy_with_sub(&self, value_id: ValueId) -> Self {
        match self {
            Self::Static(value) => Self::Static(*value),
            Self::Dyn(val) => Self::Dyn(val.copy_with_sub(value_id)),
            Self::Index(value, index) => Self::Index(
                value.copy_with_sub(value_id).into(),
                index.copy_with_sub(value_id).into(),
            ),
            Self::Pending(_) => panic!("this should not be called on a pending value"),
            Self::ExprMap(map) => Self::ExprMap(
                map.iter()
                    .map(|(k, v)| (k.clone(), v.copy_with_sub(value_id)))
                    .collect(),
            ),
            Self::ExprList(list) => Self::ExprList(list.iter().map(|val| val.copy_with_sub(value_id)).collect()),
            Self::Negative(val) => Self::Negative(val.copy_with_sub(value_id).into()),
            Self::Op(lhs, rhs, op) => Self::Op(
                lhs.copy_with_sub(value_id).into(),
                rhs.copy_with_sub(value_id).into(),
                *op,
            ),
            Self::Not(val) => Self::Not(val.copy_with_sub(value_id).into()),
            Self::Equality(lhs, rhs, eq) => Self::Equality(
                lhs.copy_with_sub(value_id).into(),
                rhs.copy_with_sub(value_id).into(),
                *eq,
            ),
            Self::Empty => Self::Empty,
        }
    }

    fn get(&self, path: Path<'_>, value_id: ValueId) -> Option<EvalValue<'bp>> {
        match self {
            EvalValue::Dyn(value) => Some(EvalValue::Dyn(
                value.as_state().and_then(|state| state.state_get(path, value_id))?,
            )),
            EvalValue::Index(value, _) => value.get(path, value_id),
            EvalValue::Pending(_) => {
                unreachable!("pending values are resolved by the scope and should never exist here")
            }
            EvalValue::ExprMap(map) => match path {
                Path::Key(key) => Some(map.get(key)?.copy_with_sub(value_id)),
                Path::Index(_) => None,
            },
            EvalValue::ExprList(list) => match path {
                Path::Index(idx) => Some(list.get(idx)?.copy_with_sub(value_id)),
                Path::Key(_) => None,
            },
            EvalValue::Static(_)
            | EvalValue::Negative(_)
            | EvalValue::Op(_, _, _)
            | EvalValue::Not(_)
            | EvalValue::Equality(_, _, _)
            | EvalValue::Empty => None,
        }
    }

    /// Downgrade andy ValueRef to PendingValue
    fn inner_downgrade(&self) -> Self {
        match self {
            Self::Static(val) => Self::Static(*val),
            Self::Pending(val) => Self::Pending(*val),
            Self::Dyn(val) => Self::Pending(val.to_pending()),
            Self::Index(val, index) => Self::Index(val.inner_downgrade().into(), index.inner_downgrade().into()),
            Self::ExprMap(map) => {
                let map = map
                    .iter()
                    .map(|(key, val)| (key.clone(), val.inner_downgrade()))
                    .collect();
                Self::ExprMap(map)
            }
            Self::ExprList(list) => {
                let list = list.iter().map(Self::inner_downgrade).collect();
                Self::ExprList(list)
            }
            Self::Negative(val) => Self::Negative(val.inner_downgrade().into()),
            Self::Op(lhs, rhs, op) => Self::Op(lhs.inner_downgrade().into(), rhs.inner_downgrade().into(), *op),
            Self::Not(val) => Self::Not(val.inner_downgrade().into()),
            Self::Equality(lhs, rhs, eq) => {
                let lhs = lhs.inner_downgrade().into();
                let rhs = rhs.inner_downgrade().into();
                Self::Equality(lhs, rhs, *eq)
            }
            Self::Empty => Self::Empty,
        }
    }

    fn inner_upgrade(&self, value_id: ValueId) -> Self {
        match self {
            Self::Dyn(_) => unreachable!("the value was downgraded"),
            Self::Static(val) => Self::Static(*val),
            Self::Pending(val) => Self::Dyn(val.to_value(value_id)),
            Self::Index(value, index) => Self::Index(
                value.inner_upgrade(value_id).into(),
                index.inner_upgrade(value_id).into(),
            ),
            Self::ExprMap(map) => {
                let map = map
                    .iter()
                    .map(|(key, val)| (key.clone(), val.inner_upgrade(value_id)))
                    .collect();
                Self::ExprMap(map)
            }
            Self::ExprList(list) => {
                let list = list.iter().map(Self::inner_downgrade).collect();
                Self::ExprList(list)
            }
            Self::Negative(val) => Self::Negative(val.inner_upgrade(value_id).into()),
            Self::Op(lhs, rhs, op) => Self::Op(
                lhs.inner_upgrade(value_id).into(),
                rhs.inner_upgrade(value_id).into(),
                *op,
            ),
            Self::Not(val) => Self::Not(val.inner_upgrade(value_id).into()),
            Self::Equality(lhs, rhs, eq) => {
                let lhs = lhs.inner_upgrade(value_id).into();
                let rhs = rhs.inner_upgrade(value_id).into();
                Self::Equality(lhs, rhs, *eq)
            }
            Self::Empty => future_value(value_id),
        }
    }

    pub(crate) fn downgrade(&self) -> Downgraded<'bp> {
        Downgraded(self.inner_downgrade())
    }

    pub fn str_for_each<F>(&self, mut f: F)
    where
        F: FnMut(&str),
    {
        let mut wrapped_f = |s: &str| {
            f(s);
            ControlFlow::Continue(())
        };

        match self.internal_str_iter(&mut wrapped_f) {
            Some(control_flow) => control_flow,
            None => ControlFlow::Break(()),
        };
    }

    pub fn str_iter<F>(&self, mut f: F) -> ControlFlow<()>
    where
        F: FnMut(&str) -> ControlFlow<()>,
    {
        match self.internal_str_iter(&mut f) {
            Some(control_flow) => control_flow,
            None => ControlFlow::Break(()),
        }
    }

    fn internal_str_iter<F>(&self, f: &mut F) -> Option<ControlFlow<()>>
    where
        F: FnMut(&str) -> ControlFlow<()>,
    {
        let val = match self {
            EvalValue::ExprList(list) => {
                for value in list.iter() {
                    value.internal_str_iter(f)?;
                }
                ControlFlow::Continue(())
            }
            EvalValue::Static(val) => {
                let s = val.to_common_str();
                let s = s.as_ref();
                f(s)
            }
            EvalValue::Dyn(val) => {
                let state = val.as_state()?;
                let common = state.to_common()?;
                let s = common.to_common_str();
                let s = s.as_ref();
                f(s)
            }
            EvalValue::Index(val, _) => val.internal_str_iter(f)?,
            _ => {
                let val = self.load_common_val()?;
                let val = val.to_common()?;
                f(val.to_common_str().as_ref())
            }
        };

        Some(val)
    }

    /// Load a common value OR a shared state that can become a common value.
    /// This is only used by templates and not widgets / elements.
    pub fn load_common_val(&self) -> Option<Either<'_>> {
        match self {
            EvalValue::Static(val) => Some(Either::Static(*val)),
            EvalValue::Dyn(val) => Some(Either::Dyn(val.as_state()?)),
            EvalValue::Index(val, _) => val.load_common_val(),
            EvalValue::Pending(_) => None,
            EvalValue::ExprMap(_) => None,
            EvalValue::ExprList(_) => None,

            // Operations
            EvalValue::Negative(expr) => expr.load_number().map(|n| -n).map(Into::into),
            EvalValue::Op(lhs, rhs, op) => {
                let lhs = lhs.load_number()?;
                let rhs = rhs.load_number()?;
                let res = match *op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => lhs / rhs,
                    Op::Mod => lhs % rhs,
                };
                Some(res.into())
            }

            // Equality
            EvalValue::Not(val) => Some(CommonVal::from(!val.load_bool()).into()),
            EvalValue::Equality(lhs, rhs, eq) => {
                let b = match eq {
                    Equality::Eq => {
                        let lhs = lhs.load_common_val()?;
                        let rhs = rhs.load_common_val()?;
                        lhs.to_common()? == rhs.to_common()?
                    }
                    Equality::NotEq => {
                        let lhs = lhs.load_common_val()?;
                        let rhs = rhs.load_common_val()?;
                        lhs.to_common()? != rhs.to_common()?
                    }
                    Equality::And => lhs.load_bool() && rhs.load_bool(),
                    Equality::Or => lhs.load_bool() || rhs.load_bool(),
                    Equality::Gt => lhs.load_number()? > rhs.load_number()?,
                    Equality::Gte => lhs.load_number()? >= rhs.load_number()?,
                    Equality::Lt => lhs.load_number()? < rhs.load_number()?,
                    Equality::Lte => lhs.load_number()? <= rhs.load_number()?,
                };
                Some(CommonVal::from(b).into())
            }
            EvalValue::Empty => None,
        }
    }

    pub(crate) fn load_bool(&self) -> bool {
        let Some(value) = self.load_common_val() else { return false };
        match value {
            Either::Static(val) => val.to_bool(),
            Either::Dyn(state) => (*state).to_common().map(|v| v.to_bool()).unwrap_or(false),
        }
    }

    pub(crate) fn load_number(&self) -> Option<Number> {
        let val = self.load_common_val()?;
        match val {
            Either::Static(val) => val.to_number(),
            Either::Dyn(state) => (*state).to_common().and_then(|v| v.to_number()),
        }
    }

    // Load a value from an expression.
    // If the value is `EvalValue::Dyn` it can possible circumvent the need
    // for `CommonVal`. However if the value originates from a template rather than
    // state, then it has to go through `CommonVal`.
    //
    // For this reason the `CommonVal: From<T>` bound is required.
    pub(crate) fn load<T>(&self) -> Option<T>
    where
        T: 'static,
        T: for<'a> TryFrom<CommonVal<'a>>,
        T: Copy + PartialEq,
    {
        match self {
            EvalValue::Static(p) => (*p).try_into().ok(),
            EvalValue::Dyn(val) => match val.value::<T>() {
                Some(value) => value.try_as_ref().copied(),
                None => val.as_state()?.to_common()?.try_into().ok(),
            },
            EvalValue::Index(val, _) => val.load::<T>(),
            EvalValue::Op(lhs, rhs, op) => {
                let lhs = lhs.load_number()?;
                let rhs = rhs.load_number()?;
                let res = match *op {
                    Op::Add => Some(lhs + rhs),
                    Op::Sub => Some(lhs - rhs),
                    Op::Mul => Some(lhs * rhs),
                    Op::Div => Some(lhs / rhs),
                    Op::Mod => Some(lhs % rhs),
                };

                T::try_from(res?.into()).ok()
            }
            expr @ EvalValue::Negative(_) => {
                let val = expr.load_number()?;
                T::try_from(val.into()).ok()
            }
            EvalValue::Not(expr) => {
                let val = !expr.load_bool();
                T::try_from(val.into()).ok()
            }
            s @ EvalValue::Equality(..) => {
                let val = CommonVal::Bool(s.load_bool());
                T::try_from(val).ok()
            }
            EvalValue::Empty => None,
            e => panic!("{e:?}"),
        }
    }

    // If the eval value contains an index this value would
    // be subject to change if the index it self was updated
    pub(crate) fn contains_index(&self) -> bool {
        match self {
            Self::Index(..) => true,
            Self::ExprList(list) => list.iter().any(Self::contains_index),
            Self::ExprMap(_) => todo!(),
            _ => false,
        }
    }
}

impl From<PendingValue> for EvalValue<'_> {
    fn from(value: PendingValue) -> Self {
        Self::Pending(value)
    }
}

impl<'bp> From<CommonVal<'bp>> for EvalValue<'bp> {
    fn from(value: CommonVal<'bp>) -> Self {
        Self::Static(value)
    }
}

impl From<ValueRef> for EvalValue<'_> {
    fn from(value: ValueRef) -> Self {
        Self::Dyn(value)
    }
}

impl<'a> TryFrom<&EvalValue<'a>> for &'a str {
    type Error = ();

    fn try_from(value: &EvalValue<'a>) -> Result<Self, Self::Error> {
        match value {
            EvalValue::Static(CommonVal::Str(s)) => Ok(s),
            _ => Err(()),
        }
    }
}

struct ValueResolver<'bp> {
    globals: &'bp Globals,
    scope_offset: Option<usize>,
    value_id: ValueId,
}

impl<'bp> ValueResolver<'bp> {
    fn new(globals: &'bp Globals, value_id: ValueId) -> Self {
        Self {
            scope_offset: None,
            globals,
            value_id,
        }
    }

    fn reset_offset(&self) -> Self {
        Self {
            scope_offset: None,
            globals: self.globals,
            value_id: self.value_id,
        }
    }

    // NOTE
    // Eval values will never be pending here as pending values are resolved by the scope lookup.
    // This should probably be expressed with the type system instead but since downgraded values
    // are recursive a wrapper won't do.
    fn lookup(&mut self, expr: &'bp Expression, scope: &Scope<'bp>, states: &States) -> EvalValue<'bp> {
        match expr {
            Expression::Ident(ident) => {
                let lookup = ScopeLookup::new(&**ident, self.value_id);

                let Some(val) = scope.get(lookup, &mut self.scope_offset, states) else {
                    match self.globals.get(ident) {
                        Some(expr) => return self.reset_offset().resolve(expr, scope, states),
                        None => return future_value(self.value_id),
                    }
                };

                val
            }
            Expression::Index(lhs, rhs) => {
                // -----------------------------------------------------------------------------
                //   - Index -
                // -----------------------------------------------------------------------------
                let rhs = self.reset_offset().resolve(rhs, scope, states);
                if rhs == EvalValue::Empty {
                    // No need to register a future value here as that is already
                    // done when trying to resolve the `rhs`.
                    return EvalValue::Empty;
                }
                let Some(common_val) = rhs.load_common_val() else { return future_value(self.value_id) };
                let Some(path) = common_val.load_path() else { return future_value(self.value_id) };

                // -----------------------------------------------------------------------------
                //   - Static list -
                // -----------------------------------------------------------------------------
                if let (Expression::List(list), Path::Index(i)) = (lhs.as_ref(), path) {
                    let Some(expr) = list.get(i) else { return future_value(self.value_id) };
                    let value = self.reset_offset().resolve(expr, scope, states);
                    drop(common_val);
                    return EvalValue::Index(value.into(), rhs.into());
                }

                // -----------------------------------------------------------------------------
                //   - Static map -
                // -----------------------------------------------------------------------------
                if let (Expression::Map(map), Path::Key(key)) = (lhs.as_ref(), path) {
                    let Some(expr) = map.get(key) else { return future_value(self.value_id) };
                    let value = self.reset_offset().resolve(expr, scope, states);
                    drop(common_val);
                    return EvalValue::Index(value.into(), rhs.into());
                }

                let lhs = self.resolve(lhs, scope, states);
                match &lhs {
                    EvalValue::Index(val, _) => match val.get(path, self.value_id) {
                        Some(val) => {
                            drop(common_val);
                            EvalValue::Index(val.into(), EvalValue::Index(lhs.into(), rhs.into()).into())
                        }
                        None => future_value(self.value_id),
                    },
                    EvalValue::Dyn(value_ref) => {
                        match value_ref
                            .as_state()
                            .and_then(|state| state.state_get(path, self.value_id))
                        {
                            Some(value) => {
                                drop(common_val);
                                EvalValue::Index(
                                    EvalValue::Dyn(value).into(),
                                    EvalValue::Index(lhs.into(), rhs.into()).into(),
                                )
                            }
                            None => future_value(self.value_id),
                        }
                    }
                    EvalValue::ExprList(_) | EvalValue::ExprMap(_) => match lhs.get(path, self.value_id) {
                        Some(val) => val,
                        None => future_value(self.value_id),
                    },
                    _ => future_value(self.value_id),
                }
            }
            _ => EvalValue::Empty,
        }
    }

    fn resolve(&mut self, expr: &'bp Expression, scope: &Scope<'bp>, states: &States) -> EvalValue<'bp> {
        use {EvalValue as V, Expression as E};

        match expr {
            // -----------------------------------------------------------------------------
            //   - Values -
            // -----------------------------------------------------------------------------
            E::Primitive(val) => V::Static((*val).into()),
            E::Str(s) => V::Static(CommonVal::Str(s)),
            E::Map(map) => {
                let inner = map
                    .iter()
                    .map(|(key, expr)| (key.clone(), self.reset_offset().resolve(expr, scope, states)))
                    .collect();
                V::ExprMap(inner)
            }
            E::List(list) => {
                let inner = list
                    .iter()
                    .map(|expr| self.reset_offset().resolve(expr, scope, states))
                    .collect();
                V::ExprList(inner)
            }

            // -----------------------------------------------------------------------------
            //   - Lookups -
            // -----------------------------------------------------------------------------
            E::Ident(_) | E::Index(..) => self.lookup(expr, scope, states),

            // -----------------------------------------------------------------------------
            //   - Conditionals -
            // -----------------------------------------------------------------------------
            E::Not(expr) => V::Not(self.resolve(expr, scope, states).into()),
            E::Equality(lhs, rhs, eq) => V::Equality(
                self.reset_offset().resolve(lhs, scope, states).into(),
                self.reset_offset().resolve(rhs, scope, states).into(),
                *eq,
            ),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            E::Negative(expr) => V::Negative(self.resolve(expr, scope, states).into()),

            E::Op(lhs, rhs, op) => {
                let lhs = self.reset_offset().resolve(lhs, scope, states);
                let rhs = self.reset_offset().resolve(rhs, scope, states);
                V::Op(lhs.into(), rhs.into(), *op)
            }

            // -----------------------------------------------------------------------------
            //   - Function call -
            // -----------------------------------------------------------------------------
            E::Call { fun: _, args: _ } => todo!(),
        }
    }
}

pub(crate) fn eval<'bp>(
    expr: &'bp Expression,
    globals: &'bp Globals,
    scope: &Scope<'bp>,
    states: &States,
    value_id: impl Into<ValueId>,
) -> Value<'bp, EvalValue<'bp>> {
    let value_id = value_id.into();
    let value = ValueResolver::new(globals, value_id).resolve(expr, scope, states);
    Value::new(value, Some(expr))
}

pub(crate) fn eval_collection<'bp>(
    expr: &'bp Expression,
    globals: &'bp Globals,
    scope: &Scope<'bp>,
    states: &States,
    value_id: ValueId,
) -> Value<'bp, Collection<'bp>> {
    let value = ValueResolver::new(globals, value_id).resolve(expr, scope, states);

    let collection = match value {
        EvalValue::Dyn(val) => Collection::Dyn(val),
        EvalValue::ExprList(list) => Collection::Static(list),
        EvalValue::Index(list, rhs) => match *list {
            EvalValue::Dyn(val) => Collection::Index(Collection::Dyn(val).into(), rhs),
            EvalValue::ExprList(list) => Collection::Index(Collection::Static(list).into(), rhs),
            _ => Collection::Future,
        },
        _ => Collection::Future,
    };

    Value::new(collection, Some(expr))
}

#[cfg(test)]
mod test {

    use anathema_state::{List, Map, Value};
    use anathema_templates::expressions::{
        add, and, eq, greater_than, greater_than_equal, ident, index, less_than, less_than_equal, mul, neg, not, num,
        or, strlit, sub,
    };

    use crate::testing::ScopedTest;

    #[test]
    fn index_lookup_on_lists_of_lists() {
        let mut numbers = List::empty();
        numbers.push_back(123u32);

        let mut lists = List::<List<_>>::empty();
        lists.push_back(numbers);

        ScopedTest::new()
            .with_value("a", lists)
            // Subtract 115 from a[0][0]
            .with_expr(sub(index(index(ident("a"), num(0)), num(0)), num(115)))
            .eval(|value| {
                let val = value.load::<u32>().unwrap();
                assert_eq!(val, 8);
            });
    }

    #[test]
    fn index_lookup_on_lists_of_maps() {
        let mut map = Map::empty();
        map.insert("val", 1u32);

        let mut lists = List::<Value<Map<u32>>>::empty();
        lists.push_back(map);

        ScopedTest::new()
            .with_value("list", lists)
            .with_expr(add(index(index(ident("list"), num(0)), strlit("val")), num(1)))
            .eval(|value| {
                let val = value.load::<u32>().unwrap();
                assert_eq!(val, 2);
            });
    }

    #[test]
    fn simple_lookup() {
        let mut t = ScopedTest::new().with_value("a", 1u32).with_expr(ident("a"));

        t.eval(|value| {
            let val = value.load::<u32>().unwrap();
            assert_eq!(val, 1);
        });
    }

    #[test]
    fn dyn_add() {
        let mut t = ScopedTest::new()
            .with_value("a", 1u32)
            .with_value("b", 2u32)
            .with_expr(mul(add(ident("b"), ident("b")), ident("b")));

        t.eval(|value| {
            let val = value.load::<u32>().unwrap();
            assert_eq!(val, 8);
        });
    }

    #[test]
    fn dyn_neg() {
        ScopedTest::new()
            .with_value("a", 2i32)
            .with_expr(neg(ident("a")))
            .eval(|value| {
                let val = value.load::<i32>().unwrap();
                assert_eq!(val, -2);
            });
    }

    #[test]
    fn dyn_not() {
        ScopedTest::new()
            .with_value("a", true)
            .with_expr(not(ident("a")))
            .eval(|value| {
                let val = value.load::<bool>().unwrap();
                assert!(!val);
            });
    }

    #[test]
    fn dyn_not_no_val() {
        ScopedTest::<bool, _>::new().with_expr(not(ident("a"))).eval(|value| {
            let val = value.load::<bool>().unwrap();
            assert!(val);
        });
    }

    #[test]
    fn bool_eval() {
        ScopedTest::new()
            .with_value("a", true)
            .with_expr(not(not(ident("a"))))
            .eval(|value| {
                let val = value.load::<bool>().unwrap();
                assert!(val);
            });
    }

    #[test]
    fn equality() {
        ScopedTest::new()
            .with_value("a", 1)
            .with_value("b", 2)
            .with_expr(eq(add(ident("a"), num(1)), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn str_equality() {
        ScopedTest::new()
            .with_value("a", "lark")
            .with_value("b", "lark")
            .with_expr(eq(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn and_expr() {
        ScopedTest::new()
            .with_value("a", true)
            .with_value("b", true)
            .with_expr(and(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn or_expr() {
        ScopedTest::new()
            .with_value("a", false)
            .with_value("b", true)
            .with_expr(or(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn greater_expr() {
        ScopedTest::new()
            .with_value("a", 2)
            .with_value("b", 1)
            .with_expr(greater_than(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn greater_equal_expr() {
        ScopedTest::new()
            .with_value("a", 2)
            .with_value("b", 2)
            .with_expr(greater_than_equal(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn lesser_expr() {
        ScopedTest::new()
            .with_value("a", 1)
            .with_value("b", 2)
            .with_expr(less_than(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn lesser_equal_expr() {
        ScopedTest::new()
            .with_value("a", 2)
            .with_value("b", 2)
            .with_expr(less_than_equal(ident("a"), ident("b")))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }
}
