use std::collections::HashMap;
use std::ops::{ControlFlow, Deref};
use std::rc::Rc;

use anathema_state::{register_future, CommonVal, Number, Path, PendingValue, SharedState, State, States, ValueRef};
use anathema_templates::expressions::{Equality, Op};
use anathema_templates::Expression;

use crate::scope::{Scope, ScopeLookup};
use crate::values::{Collection, ValueId};
use crate::Value;

pub enum CommonRef<'a, 'b> {
    Owned(CommonVal<'b>),
    Borrowed(&'b CommonVal<'a>),
}

impl<'a, 'b> Deref for CommonRef<'a, 'b> {
    type Target = CommonVal<'b>;

    fn deref(&self) -> &Self::Target {
        match self {
            CommonRef::Owned(o) => o,
            CommonRef::Borrowed(b) => b,
        }
    }
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

    pub fn to_common<'b>(&'b self) -> Option<CommonRef<'a, 'b>> {
        match self {
            Either::Static(val) => Some(CommonRef::Borrowed(val)),
            Either::Dyn(state) => state.to_common().map(CommonRef::Owned),
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
    pub(crate) fn upgrade(&self, value_id: Option<ValueId>) -> EvalValue<'bp> {
        self.0.inner_upgrade(value_id)
    }
}

#[derive(Debug)]
pub enum EvalValue<'bp> {
    Static(CommonVal<'bp>),
    Dyn(ValueRef),
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
    /// Downgrade andy ValueRef to PendingValue
    fn inner_downgrade(&self) -> Self {
        match self {
            Self::Static(val) => Self::Static(*val),
            Self::Pending(val) => Self::Pending(*val),
            Self::Dyn(val) => Self::Pending(val.to_pending()),
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

    fn inner_upgrade(&self, value_id: Option<ValueId>) -> Self {
        match self {
            Self::Static(val) => Self::Static(*val),
            Self::Pending(val) => match value_id {
                Some(value_id) => Self::Dyn(val.to_value(value_id)),
                None => Self::Pending(*val),
            },
            Self::Dyn(_val) => unreachable!("the value was downgraded"),
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
            Self::Empty => {
                if let Some(value_id) = value_id {
                    register_future(value_id);
                }
                Self::Empty
            }
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
            _ => {
                let val = self.load_common_val()?;
                let val = val.to_common()?;
                f(val.to_common_str().as_ref())
            }
        };

        Some(val)
    }

    /// Load a common value OR a shared state... that can become
    /// a common value...
    /// This is only used by template
    pub fn load_common_val(&self) -> Option<Either<'_>> {
        match self {
            EvalValue::Static(val) => Some(Either::Static(*val)),
            EvalValue::Dyn(val) => Some(Either::Dyn(val.as_state()?)),
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
                        *lhs.to_common()? == *rhs.to_common()?
                        // lhs.load_common_val()? == rhs.load_common_val()?
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

    pub fn load_bool(&self) -> bool {
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

    /// Load a value from an expression.
    /// If the value is `EvalValue::Dyn` it can possible circumvent the need
    /// for `CommonVal`. However if the value originates from a template rather than
    /// state, then it has to go through `CommonVal`.
    ///
    /// For this reason the `CommonVal: From<T>` bound is required.
    pub fn load<T>(&self) -> Option<T>
    where
        T: 'static,
        T: for<'a> TryFrom<CommonVal<'a>>,
        T: Copy + PartialEq,
    {
        match self {
            EvalValue::Static(p) => (*p).try_into().ok(),
            EvalValue::Dyn(val) => match val.value::<T>() {
                Some(value) => Some(*value),
                None => val.as_state()?.to_common()?.try_into().ok(),
            },
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
            EvalValue::Static(p) => match p {
                CommonVal::Str(s) => Ok(s),
                _ => Err(())
            },
            _ => Err(()),
        }
    }
}

trait Resolver {
    type Output<'bp>;

    fn resolve<'bp>(
        &mut self,
        expr: &'bp Expression,
        scope: &Scope<'bp>,
        states: &States,
        value_id: Option<ValueId>,
    ) -> Self::Output<'bp>;

    fn lookup<'bp>(
        &mut self,
        expr: &'bp Expression,
        scope: &Scope<'bp>,
        states: &States,
        value_id: Option<ValueId>,
    ) -> Self::Output<'bp>;
}

pub struct ValueResolver {
    scope_offset: Option<usize>,
}

impl ValueResolver {
    pub fn new() -> Self {
        Self { scope_offset: None }
    }
}

impl Resolver for ValueResolver {
    type Output<'bp> = EvalValue<'bp>;

    fn lookup<'bp>(
        &mut self,
        expr: &'bp Expression,
        scope: &Scope<'bp>,
        states: &States,
        value_id: Option<ValueId>,
    ) -> Self::Output<'bp> {
        match expr {
            Expression::Ident(ident) => {
                let lookup = ScopeLookup::new(&**ident, value_id);

                let Some(val) = scope.get(lookup, &mut self.scope_offset, states) else {
                    if let Some(id) = value_id {
                        register_future(id);
                    }
                    return EvalValue::Empty;
                };
                val
            }
            Expression::Dot(lhs, rhs) => {
                // The `lhs` can never resolve to anything but a Pending value
                // as there should be no subscription to changes on anything but the rhs
                let lhs = self.resolve(lhs, scope, states, None);
                match lhs {
                    EvalValue::Pending(pending_val) => {
                        let Expression::Ident(key) = &**rhs else {
                            if let Some(id) = value_id {
                                register_future(id);
                            }
                            return EvalValue::Empty;
                        };

                        match value_id {
                            Some(id) => match pending_val.as_state(|state| state.state_get(Path::Key(key), id)) {
                                Some(val) => EvalValue::Dyn(val),
                                None => {
                                    if let Some(id) = value_id {
                                        register_future(id);
                                    }
                                    EvalValue::Empty
                                }
                            },
                            None => match pending_val.as_state(|state| state.state_lookup(Path::Key(key))) {
                                Some(val) => EvalValue::Pending(val),
                                None => {
                                    if let Some(id) = value_id {
                                        register_future(id);
                                    }
                                    EvalValue::Empty
                                }
                            },
                        }
                    }
                    _ => {
                        if let Some(id) = value_id {
                            register_future(id);
                        }
                        EvalValue::Empty
                    }
                }
            }
            Expression::Index(lhs, index) => {
                let lhs = self.resolve(lhs, scope, states, None);
                match lhs {
                    EvalValue::Pending(pending_val) => {
                        let key = match &**index {
                            Expression::Str(key) => Path::Key(key),
                            _ => match self.resolve(index, scope, states, value_id) {
                                EvalValue::Static(CommonVal::Int(index)) => Path::Index(index as usize),
                                _ => return EvalValue::Empty,
                            },
                        };
                        match value_id {
                            Some(id) => match pending_val.as_state(|state| state.state_get(key, id)) {
                                Some(val) => EvalValue::Dyn(val),
                                None => {
                                    if let Some(id) = value_id {
                                        register_future(id);
                                    }
                                    EvalValue::Empty
                                }
                            },
                            None => match pending_val.as_state(|state| state.state_lookup(key)) {
                                Some(val) => EvalValue::Pending(val),
                                None => {
                                    if let Some(id) = value_id {
                                        register_future(id);
                                    }
                                    EvalValue::Empty
                                }
                            },
                        }
                    }
                    _ => {
                        if let Some(id) = value_id {
                            register_future(id);
                        }
                        EvalValue::Empty
                    }
                }
            }
            _ => EvalValue::Empty,
        }
    }

    fn resolve<'bp>(
        &mut self,
        expr: &'bp Expression,
        scope: &Scope<'bp>,
        states: &States,
        value_id: Option<ValueId>,
    ) -> Self::Output<'bp> {
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
                    .map(|(key, expr)| (key.clone(), Self::new().resolve(expr, scope, states, value_id)))
                    .collect();
                V::ExprMap(inner)
            }
            E::List(list) => {
                let inner = list
                    .iter()
                    .map(|expr| Self::new().resolve(expr, scope, states, value_id))
                    .collect();
                V::ExprList(inner)
            }

            // -----------------------------------------------------------------------------
            //   - Lookups -
            // -----------------------------------------------------------------------------
            E::Ident(_) | E::Dot(..) | E::Index(..) => self.lookup(expr, scope, states, value_id),

            // -----------------------------------------------------------------------------
            //   - Conditionals -
            // -----------------------------------------------------------------------------
            E::Not(expr) => V::Not(self.resolve(expr, scope, states, value_id).into()),
            E::Equality(lhs, rhs, eq) => V::Equality(
                Self::new().resolve(lhs, scope, states, value_id).into(),
                Self::new().resolve(rhs, scope, states, value_id).into(),
                *eq,
            ),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            E::Negative(expr) => V::Negative(self.resolve(expr, scope, states, value_id).into()),

            E::Op(lhs, rhs, op) => {
                let lhs = Self::new().resolve(lhs, scope, states, value_id);
                let rhs = Self::new().resolve(rhs, scope, states, value_id);
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
    scope: &Scope<'bp>,
    states: &States,
    value_id: impl Into<ValueId>,
) -> Value<'bp, EvalValue<'bp>> {
    let value_id = value_id.into();
    let value = ValueResolver::new().resolve(expr, scope, states, Some(value_id));
    Value::new(value, Some(expr))
}

pub(crate) fn eval_collection<'bp>(
    expr: &'bp Expression,
    scope: &Scope<'bp>,
    states: &States,
    value_id: ValueId,
) -> Value<'bp, Collection<'bp>> {
    let value = ValueResolver::new().resolve(expr, scope, states, Some(value_id));
    let collection = match value {
        EvalValue::Dyn(val) => Collection::Dyn(val),
        EvalValue::ExprList(list) => Collection::Static(list),
        _ => Collection::Future,
    };

    Value::new(collection, Some(expr))
}

#[cfg(test)]
mod test {

    use anathema_state::List;
    use anathema_templates::expressions::{
        add, and, eq, greater_than, greater_than_equal, ident, index, less_than, less_than_equal, mul, neg, not, num,
        or, sub,
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
