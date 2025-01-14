use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::ops::ControlFlow;
use std::rc::Rc;

use anathema_state::{
    register_future, CommonString, CommonVal, Number, Path, PendingValue, SharedState, StateId, States, ValueRef,
};
use anathema_strings::{HString, HStrings, StrIndex, Transaction};
use anathema_templates::expressions::{Equality, Op};
use anathema_templates::{Expression, Globals};
pub(crate) use values::ValueKind;

use crate::scope::{Scope, ScopeLookup};
use crate::values::{Collection, ValueId};
use crate::{AttributeStorage, Value, WidgetId};

// pub(crate) mod resolver;
pub(crate) mod values;

pub(crate) struct ExprEvalCtx<'a, 'bp> {
    pub(crate) states: &'a States,
    pub(crate) scope: &'a Scope<'bp>,
    pub(crate) attributes: &'a AttributeStorage<'bp>,
    pub(crate) globals: &'bp Globals,
}

// TODO: name this something else (and if you suggest SomethingElse as the name you are a lark)
#[derive(Debug)]
pub enum NameThis<'a> {
    Nothing,
    Value(EvalValue<'a>),
    ResolveThisNow(&'a Expression),
}

impl<'a> From<Option<EvalValue<'a>>> for NameThis<'a> {
    fn from(value: Option<EvalValue<'a>>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::Nothing,
        }
    }
}

impl<'a> From<EvalValue<'a>> for NameThis<'a> {
    fn from(value: EvalValue<'a>) -> Self {
        Self::Value(value)
    }
}

pub enum Either {
    Static(CommonVal),
    Dyn(SharedState),
}

impl Either {
    pub fn load_bool(&self) -> bool {
        panic!("remove this once the new resolver is in place");
        // match self {
        //     Either::Static(val) => val.to_bool(),
        //     Either::Dyn(state) => state.to_common().map(|v| v.to_bool()).unwrap_or(false),
        // }
    }

    pub fn load_number(&self) -> Option<Number> {
        panic!("remove this once the new resolver is in place");
        // match self {
        //     Either::Static(val) => val.to_number(),
        //     Either::Dyn(state) => state.to_common().and_then(|v| v.to_number()),
        // }
    }

    pub fn to_common(&self) -> Option<CommonVal> {
        panic!("remove this once the new resolver is in place");
        // match self {
        //     Either::Static(val) => Some(*val),
        //     Either::Dyn(state) => state.to_common(),
        // }
    }
}

impl From<CommonVal> for Either {
    fn from(value: CommonVal) -> Self {
        Self::Static(value)
    }
}

impl From<Number> for Either {
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
    Static(CommonVal),
    Dyn(ValueRef),
    State(StateId),
    ComponentAttributes(WidgetId),
    Index(Box<Self>, Box<Self>),
    /// Pending value is used for collections
    /// and traversing state, as a means
    /// to access state without subscribing to it
    Pending(PendingValue),
    // Map(HashMap<&'bp str, EvalValue<'bp>>),
    ExprList(&'bp [Expression]),
    List(Box<[EvalValue<'bp>]>),
    String(StrIndex),
    Map(&'bp HashMap<String, Expression>),

    // Operations
    Negative(Box<Self>),
    Op(Box<Self>, Box<Self>, Op),

    // Equality
    Not(Box<Self>),
    Equality(Box<Self>, Box<Self>, Equality),

    Empty,
}

impl<'bp> EvalValue<'bp> {
    // fn copy_with_sub(&self, value_id: ValueId) -> Self {
    //     match self {
    //         Self::Static(value) => Self::Static(*value),
    //         Self::Dyn(val) => Self::Dyn(val.copy_with_sub(value_id)),
    //         Self::String(s) => Self::String(*s),
    //         Self::State(state_id) => Self::State(*state_id),
    //         Self::ComponentAttributes(component_id) => Self::ComponentAttributes(*component_id),
    //         Self::Index(value, index) => Self::Index(
    //             value.copy_with_sub(value_id).into(),
    //             index.copy_with_sub(value_id).into(),
    //         ),
    //         Self::Pending(_) => panic!("this should not be called on a pending value"),
    //         // Self::Map(map) => Self::Map(
    //         //     map.iter()
    //         //         .map(|(k, v)| (k.clone(), v.copy_with_sub(value_id)))
    //         //         .collect(),
    //         // ),
    //         Self::ExprList(list) => Self::ExprList(list),
    //         Self::List(_) => panic!("copy should not be done on evaluated lists"),
    //         Self::Map(map) => Self::Map(map),
    //         Self::Negative(val) => Self::Negative(val.copy_with_sub(value_id).into()),
    //         Self::Op(lhs, rhs, op) => Self::Op(
    //             lhs.copy_with_sub(value_id).into(),
    //             rhs.copy_with_sub(value_id).into(),
    //             *op,
    //         ),
    //         Self::Not(val) => Self::Not(val.copy_with_sub(value_id).into()),
    //         Self::Equality(lhs, rhs, eq) => Self::Equality(
    //             lhs.copy_with_sub(value_id).into(),
    //             rhs.copy_with_sub(value_id).into(),
    //             *eq,
    //         ),
    //         Self::Empty => Self::Empty,
    //     }
    // }

    // This is only used by the expression evaluation `Expression::Index`
    // If the lhs is a list of expression, the selected expression has to be evaluated
    // in this function.
    pub(crate) fn get(
        &self,
        path: Path<'_>,
        value_id: ValueId,
        states: &States,
        attribs: &AttributeStorage<'bp>,
    ) -> NameThis<'bp> {
        panic!("remove this once the new resolver is in place");
        // match self {
        //     EvalValue::ExprList(expressions) => match path {
        //         Path::Index(idx) if idx >= expressions.len() => NameThis::Nothing,
        //         Path::Index(idx) => {
        //             let expr = &expressions[idx];
        //             NameThis::ResolveThisNow(expr)
        //         }
        //         Path::Key(_) => NameThis::Nothing,
        //     },
        //     EvalValue::List(list) => match path {
        //         Path::Index(idx) if idx >= list.len() => NameThis::Nothing,
        //         Path::Index(idx) => {
        //             panic!("this should only ever happen when resolving a collection right?");
        //             // NameThis::Value(list[idx].copy_with_sub(value_id)),
        //         }
        //         Path::Key(_) => NameThis::Nothing,
        //     },
        //     EvalValue::Map(map) => match path {
        //         Path::Key(key) => match map.get(key) {
        //             Some(expr) => NameThis::ResolveThisNow(expr),
        //             None => NameThis::Nothing,
        //         },
        //         Path::Index(idx) => NameThis::Nothing,
        //     },
        //     EvalValue::Dyn(value) => value
        //         .as_state()
        //         .and_then(|state| state.state_get(path, value_id))
        //         .map(EvalValue::Dyn)
        //         .into(),
        //     EvalValue::Index(value, _) => value.get(path, value_id, states, attribs),
        //     EvalValue::State(id) => {
        //         // states
        //         // .get(*id)
        //         // .and_then(|state| state.state_get(path, value_id).map(EvalValue::Dyn))
        //         // .into();
        //         panic!()
        //     }
        //     EvalValue::ComponentAttributes(id) => {
        //         let Some(attributes) = attribs.try_get(*id) else { return NameThis::Nothing };
        //         let value = match path {
        //             Path::Key(key) => match attributes.get_val(key) {
        //                 Some(val) => val,
        //                 None => return NameThis::Nothing,
        //             },
        //             Path::Index(_) => unreachable!("attributes are not indexed by numbers"),
        //         };
        //         panic!("figure this out");
        //         // NameThis::Value(value.copy_with_sub(value_id).into())
        //     }
        //     EvalValue::Pending(_) => {
        //         unreachable!("pending values are resolved by the scope and should never exist here")
        //     }
        //     // EvalValue::Map(map) => match path {
        //     //     Path::Key(key) => panic!("see expression list 2, do that here"), //Some(map.get(key).copy_with_sub(value_id)),
        //     //     Path::Index(_) => NameThis::Nothing,
        //     // },
        //     EvalValue::Static(_)
        //     | EvalValue::Negative(_)
        //     | EvalValue::Op(_, _, _)
        //     | EvalValue::Not(_)
        //     | EvalValue::Equality(_, _, _)
        //     | EvalValue::String(_)
        //     | EvalValue::Empty => NameThis::Nothing,
        // }
    }

    /// Downgrade any `ValueRef` to `PendingValue`
    fn inner_downgrade(&self) -> Self {
        match self {
            Self::Static(val) => Self::Static(*val),
            Self::Pending(val) => Self::Pending(*val),
            Self::String(val) => Self::String(*val),
            Self::Dyn(val) => Self::Pending(val.to_pending()),
            Self::State(id) => Self::State(*id),
            Self::ComponentAttributes(id) => Self::ComponentAttributes(*id),
            Self::Index(val, index) => Self::Index(val.inner_downgrade().into(), index.inner_downgrade().into()),
            // Self::Map(map) => {
            //     let map = map
            //         .iter()
            //         .map(|(key, val)| (key.clone(), val.inner_downgrade()))
            //         .collect();
            //     Self::Map(map)
            // }
            Self::ExprList(list) => Self::ExprList(list),
            Self::List(list) => Self::List(list.iter().map(|val| val.inner_downgrade()).collect()),
            Self::Map(map) => Self::Map(map),
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
            Self::String(val) => Self::String(*val),
            Self::State(id) => Self::State(*id),
            Self::ComponentAttributes(id) => Self::ComponentAttributes(*id),
            Self::Pending(val) => panic!(),//Self::Dyn(val.subscribe(value_id)),
            Self::Index(value, index) => Self::Index(
                value.inner_upgrade(value_id).into(),
                index.inner_upgrade(value_id).into(),
            ),
            // Self::Map(map) => {
            //     let map = map
            //         .iter()
            //         .map(|(key, val)| (key.clone(), val.inner_upgrade(value_id)))
            //         .collect();
            //     Self::Map(map)
            // }
            Self::ExprList(list) => Self::ExprList(list),
            Self::List(list) => Self::List(list.iter().map(|val| val.inner_upgrade(value_id)).collect()),
            Self::Map(map) => Self::Map(map),
            // Self::ExprList(list) => {
            //     let list = list.iter().map(Self::inner_downgrade).collect();
            //     Self::ExprList(list)
            // }
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
            Self::Empty => Self::Empty,
        }
    }

    pub(crate) fn downgrade(&self) -> Downgraded<'bp> {
        Downgraded(self.inner_downgrade())
    }

    fn to_hoppstr(&self, tx: &mut Transaction<'_, 'bp>) {
        match self {
            EvalValue::List(list) => {
                for value in list.iter() {
                    value.to_hoppstr(tx);
                }
            }
            EvalValue::Static(val) => {
                write!(tx, "{val}");
            }
            EvalValue::Dyn(val) => {
                panic!("should there ever be a dyn value?");
                // let Some(state) = val.as_state() else { return };
                // let Some(common) = state.to_common() else { return };
                // let s = common.to_common_str();
                // let s = s.as_ref();
                // write!(tx, "{s}");
            }
            EvalValue::Index(val, _) => val.to_hoppstr(tx),
            _ => {
                panic!("what do we do here?");
                // let Some(val) = self.load_common_val() else { return };
                // let Some(val) = val.to_common() else { return };
                // write!(tx, "{}", val.to_common_str().as_ref());
            }
        }
    }

    /// Load a common value OR a shared state that can become a common value.
    /// This is only used by templates and not widgets / elements.
    pub fn load_common_val(&self) -> Option<Either> {
        match self {
            EvalValue::Static(val) => Some(Either::Static(*val)),
            EvalValue::Dyn(val) => Some(Either::Dyn(val.as_state()?)),
            EvalValue::String(val) => panic!("figure out what to do here"),
            EvalValue::State(_) => panic!(),
            EvalValue::ComponentAttributes(_) => None, // There should be no instance where attributes is a single value
            EvalValue::Index(val, _) => val.load_common_val(),
            EvalValue::Pending(_) => None,
            EvalValue::Map(_) => None,
            EvalValue::ExprList(_) => None,
            EvalValue::List(_) => None,

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
                    // Equality::And => lhs.load_bool() && rhs.load_bool(),
                    // Equality::Or => lhs.load_bool() || rhs.load_bool(),
                    Equality::Gt => lhs.load_number()? > rhs.load_number()?,
                    Equality::Gte => lhs.load_number()? >= rhs.load_number()?,
                    Equality::Lt => lhs.load_number()? < rhs.load_number()?,
                    Equality::Lte => lhs.load_number()? <= rhs.load_number()?,
                };
                Some(CommonVal::from(b).into())
            }
            EvalValue::ExprList(_) => unreachable!(),
            EvalValue::Map(_) => unreachable!(),
            EvalValue::Empty => None,
        }
    }

    pub(crate) fn load_bool(&self) -> bool {
        panic!("remove this once the new resolver is in place");
        // let Some(value) = self.load_common_val() else { return false };
        // match value {
        //     Either::Static(val) => val.to_bool(),
        //     Either::Dyn(state) => (*state).to_common().map(|v| v.to_bool()).unwrap_or(false),
        // }
    }

    pub(crate) fn load_number(&self) -> Option<Number> {
        panic!("remove this once the new resolver is in place");
        // let val = self.load_common_val()?;
        // match val {
        //     Either::Static(val) => val.to_number(),
        //     Either::Dyn(state) => (*state).to_common().and_then(|v| v.to_number()),
        // }
    }

    pub fn load_str<'a>(&'a self, strings: &'a HStrings<'bp>) -> Option<HString<impl Iterator<Item = &str> + 'a>> {
        let EvalValue::String(hstr) = self else { return None };
        Some(strings.get(*hstr))
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
        T: TryFrom<CommonVal>,
        T: Copy + PartialEq,
    {
        panic!("remove this once the new resolver is in place");
        // match self {
        //     EvalValue::Static(p) => (*p).try_into().ok(),
        //     EvalValue::Dyn(val) => match val.value::<T>() {
        //         Some(value) => value.try_as_ref().copied(),
        //         None => val.as_state()?.to_common()?.try_into().ok(),
        //     },
        //     EvalValue::Index(val, _) => val.load::<T>(),
        //     EvalValue::Op(lhs, rhs, op) => {
        //         let lhs = lhs.load_number()?;
        //         let rhs = rhs.load_number()?;
        //         let res = match *op {
        //             Op::Add => Some(lhs + rhs),
        //             Op::Sub => Some(lhs - rhs),
        //             Op::Mul => Some(lhs * rhs),
        //             Op::Div => Some(lhs / rhs),
        //             Op::Mod => Some(lhs % rhs),
        //         };

        //         T::try_from(res?.into()).ok()
        //     }
        //     expr @ EvalValue::Negative(_) => {
        //         let val = expr.load_number()?;
        //         T::try_from(val.into()).ok()
        //     }
        //     EvalValue::Not(expr) => {
        //         let val = !expr.load_bool();
        //         T::try_from(val.into()).ok()
        //     }
        //     s @ EvalValue::Equality(..) => {
        //         let val = CommonVal::Bool(s.load_bool());
        //         T::try_from(val).ok()
        //     }
        //     EvalValue::Empty => None,
        //     e => panic!("{e:?}"),
        // }
    }
}

impl From<PendingValue> for EvalValue<'_> {
    fn from(value: PendingValue) -> Self {
        Self::Pending(value)
    }
}

impl<'bp> From<CommonVal> for EvalValue<'bp> {
    fn from(value: CommonVal) -> Self {
        Self::Static(value)
    }
}

impl From<ValueRef> for EvalValue<'_> {
    fn from(value: ValueRef) -> Self {
        Self::Dyn(value)
    }
}

pub struct Resolver<'scope, 'bp> {
    ctx: &'scope ExprEvalCtx<'scope, 'bp>,
    subscriber: ValueId,
    register_future_value: bool,
    deferred: bool,
}

impl<'scope, 'bp> Resolver<'scope, 'bp> {
    pub(crate) fn new(ctx: &'scope ExprEvalCtx<'scope, 'bp>, subscriber: ValueId, deferred: bool) -> Self {
        Self {
            ctx,
            subscriber,
            register_future_value: false,
            deferred,
        }
    }

    // pub(crate) fn root(ctx: &'scope ExprEvalCtx<'scope, 'bp>, subscriber: ValueId, deferred: bool) -> Self {
    //     Self::new(ctx, subscriber, deferred)
    // }

    pub(crate) fn resolve(&mut self, expression: &'bp Expression, strings: &mut HStrings<'bp>) -> EvalValue<'bp> {
        match expression {
            // -----------------------------------------------------------------------------
            //   - Values -
            // -----------------------------------------------------------------------------
            &Expression::Primitive(val) => EvalValue::Static(val.into()),
            Expression::Str(s) => {
                let s = strings.insert_with(|tx| tx.add_slice(s));
                EvalValue::String(s)
            }
            Expression::Map(map) => EvalValue::Map(map),
            Expression::List(list) if self.deferred => EvalValue::ExprList(list),
            Expression::List(list) => {
                let inner = list.iter().map(|expr| self.resolve(expr, strings)).collect();
                EvalValue::List(inner)
            }
            Expression::TextSegments(segments) => {
                let inner = segments
                    .iter()
                    .map(|expr| self.resolve(expr, strings))
                    .collect::<Vec<_>>();

                let s = strings.insert_with(|tx| {
                    for i in inner {
                        i.to_hoppstr(tx);
                    }
                });

                EvalValue::String(s)
            }

            // -----------------------------------------------------------------------------
            //   - Conditionals -
            // -----------------------------------------------------------------------------
            Expression::Not(expr) => EvalValue::Not(self.resolve(expr, strings).into()),
            Expression::Equality(lhs, rhs, eq) => EvalValue::Equality(
                self.resolve(lhs, strings).into(),
                self.resolve(rhs, strings).into(),
                *eq,
            ),
            Expression::LogicalOp(expression, expression1, logical_op) => todo!(),

            // -----------------------------------------------------------------------------
            //   - Lookups -
            // -----------------------------------------------------------------------------
            Expression::Ident(_) | Expression::Index(_, _) => self.lookup(expression, strings),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            Expression::Negative(expr) => EvalValue::Negative(self.resolve(expr, strings).into()),
            Expression::Op(lhs, rhs, op) => {
                let lhs = self.resolve(lhs, strings);
                let rhs = self.resolve(rhs, strings);
                EvalValue::Op(lhs.into(), rhs.into(), *op)
            }

            // -----------------------------------------------------------------------------
            //   - Either -
            // -----------------------------------------------------------------------------
            Expression::Either(lhs, rhs) => match self.resolve(lhs, strings) {
                EvalValue::Empty => self.resolve(rhs, strings),
                value => value,
            },

            // -----------------------------------------------------------------------------
            //   - Function call -
            // -----------------------------------------------------------------------------
            Expression::Call { fun: _, args: _ } => todo!(),
            Expression::Primitive(primitive) => todo!(),
        }
    }

    fn lookup(&mut self, expression: &'bp Expression, strings: &mut HStrings<'bp>) -> EvalValue<'bp> {
        match expression {
            Expression::Ident(ident) => match &**ident {
                "state" => self.ctx.scope.get_state(),
                "attributes" => self.ctx.scope.get_component_attributes(),
                path => {
                    let lookup = ScopeLookup::new(Path::from(path), self.subscriber);
                    match self.ctx.scope.get(lookup, &mut None, self.ctx.states) {
                        NameThis::Nothing => {
                            self.register_future_value = true;
                            EvalValue::Empty
                        }
                        NameThis::Value(eval_value) => eval_value,
                        NameThis::ResolveThisNow(expr) => self.resolve(expr, strings),
                    }
                }
            },
            Expression::Index(lhs, rhs) => {
                let value = self.resolve(lhs, strings);

                // The RHS is always the index / ident.
                // Note that this might still be an op, e.g a + 1
                // So the expression has to be evaluated before it can be used as an index.
                //
                // Once evaluated it should be either a string or a number
                let index = match &**rhs {
                    Expression::Str(ident) => Path::from(&**ident),
                    expr => {
                        let index = self.resolve(expr, strings);
                        if let EvalValue::Empty = index {
                            self.register_future_value = true;
                            return EvalValue::Empty;
                        }
                        let index = index.load_number().unwrap().as_int() as usize;
                        Path::from(index)
                    }
                };

                let val = match value.get(index, self.subscriber, self.ctx.states, self.ctx.attributes) {
                    NameThis::Nothing => {
                        self.register_future_value = true;
                        EvalValue::Empty
                    }
                    NameThis::Value(value) => value,
                    NameThis::ResolveThisNow(expr) => self.resolve(expr, strings),
                };

                val
            }
            _ => unreachable!("lookup only handles ident and index"),
        }
    }
}

pub(crate) fn eval<'bp>(
    expr: &'bp Expression,
    ctx: &ExprEvalCtx<'_, 'bp>,
    strings: &mut HStrings<'bp>,
    value_id: impl Into<ValueId>,
) -> Value<'bp, EvalValue<'bp>> {
    let value_id = value_id.into();

    let mut resolver = Resolver::new(ctx, value_id, false);
    let value = resolver.resolve(expr, strings);

    if resolver.register_future_value {
        register_future(value_id);
    }

    Value::new(value, Some(expr))
}

pub(crate) fn eval_collection<'s, 'bp>(
    expr: &'bp Expression,
    ctx: &ExprEvalCtx<'_, 'bp>,
    strings: &mut HStrings<'bp>,
    value_id: ValueId,
) -> Value<'bp, Collection<'bp>> {
    let value_id = value_id.into();

    let mut resolver = Resolver::new(ctx, value_id, true);
    let value = resolver.resolve(expr, strings);

    if resolver.register_future_value {
        register_future(value_id);
    }

    let collection = match value {
        EvalValue::Dyn(val) => Collection::Dyn(val),
        EvalValue::ExprList(list) => Collection::Static2(list),
        EvalValue::Index(list, rhs) => match *list {
            EvalValue::Dyn(val) => Collection::Index(Collection::Dyn(val).into(), rhs),
            EvalValue::ExprList(list) => Collection::Index(Collection::Static2(list).into(), rhs),
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
        add, and, eq, greater_than, greater_than_equal, ident, index, less_than, less_than_equal, list, mul, neg, not,
        num, or, strlit, sub, text_segment,
    };

    use super::EvalValue;
    use crate::testing::ScopedTest;

    #[test]
    fn index_lookup_on_lists_of_lists() {
        let mut numbers = List::empty();
        numbers.push_back(123u32);

        let mut lists = List::<List<_>>::empty();
        lists.push_back(numbers);

        ScopedTest::new()
            .with_state_value("a", lists)
            // Subtract 115 from a[0][0]
            .with_expr(sub(
                index(index(index(ident("state"), strlit("a")), num(0)), num(0)),
                num(115),
            ))
            .eval(|value, strings| {
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
            .with_state_value("list", lists)
            .with_expr(add(
                index(index(index(ident("state"), strlit("list")), num(0)), strlit("val")),
                num(1),
            ))
            .eval(|value, strings| {
                let val = value.load::<u32>().unwrap();
                assert_eq!(val, 2);
            });
    }

    #[test]
    fn simple_lookup() {
        let mut t = ScopedTest::new()
            .with_state_value("a", 1u32)
            .with_expr(index(ident("state"), strlit("a")));

        t.eval(|value, strings| {
            let val = value.load::<u32>().unwrap();
            assert_eq!(val, 1);
        });
    }

    #[test]
    fn dyn_add() {
        let mut t = ScopedTest::new()
            .with_state_value("a", 1u32)
            .with_state_value("b", 2u32)
            .with_expr(mul(
                add(index(ident("state"), strlit("b")), index(ident("state"), strlit("b"))),
                index(ident("state"), strlit("b")),
            ));

        t.eval(|value, strings| {
            let val = value.load::<u32>().unwrap();
            assert_eq!(val, 8);
        });
    }

    #[test]
    fn dyn_neg() {
        ScopedTest::new()
            .with_state_value("a", 2i32)
            .with_expr(neg(index(ident("state"), strlit("a"))))
            .eval(|value, strings| {
                let val = value.load::<i32>().unwrap();
                assert_eq!(val, -2);
            });
    }

    #[test]
    fn dyn_not() {
        ScopedTest::new()
            .with_state_value("a", true)
            .with_expr(not(index(ident("state"), strlit("a"))))
            .eval(|value, strings| {
                let val = value.load::<bool>().unwrap();
                assert!(!val);
            });
    }

    #[test]
    fn dyn_not_no_val() {
        ScopedTest::<bool, _>::new()
            .with_expr(not(index(ident("state"), strlit("a"))))
            .eval(|value, strings| {
                let val = value.load::<bool>().unwrap();
                assert!(val);
            });
    }

    #[test]
    fn bool_eval() {
        ScopedTest::new()
            .with_state_value("a", true)
            .with_expr(not(not(index(ident("state"), strlit("a")))))
            .eval(|value, strings| {
                let val = value.load::<bool>().unwrap();
                assert!(val);
            });
    }

    #[test]
    fn equality() {
        ScopedTest::new()
            .with_state_value("a", 1)
            .with_state_value("b", 2)
            .with_expr(eq(
                add(index(ident("state"), strlit("a")), num(1)),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn str_equality() {
        ScopedTest::new()
            .with_state_value("a", "lark")
            .with_state_value("b", "lark")
            .with_expr(eq(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn and_expr() {
        ScopedTest::new()
            .with_state_value("a", true)
            .with_state_value("b", true)
            .with_expr(and(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn or_expr() {
        ScopedTest::new()
            .with_state_value("a", false)
            .with_state_value("b", true)
            .with_expr(or(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn greater_expr() {
        ScopedTest::new()
            .with_state_value("a", 2)
            .with_state_value("b", 1)
            .with_expr(greater_than(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn greater_equal_expr() {
        ScopedTest::new()
            .with_state_value("a", 2)
            .with_state_value("b", 2)
            .with_expr(greater_than_equal(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn lesser_expr() {
        ScopedTest::new()
            .with_state_value("a", 1)
            .with_state_value("b", 2)
            .with_expr(less_than(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn lesser_equal_expr() {
        ScopedTest::new()
            .with_state_value("a", 2)
            .with_state_value("b", 2)
            .with_expr(less_than_equal(
                index(ident("state"), strlit("a")),
                index(ident("state"), strlit("b")),
            ))
            .eval(|value, strings| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }

    #[test]
    fn strings() {
        ScopedTest::new()
            .with_state_value("s", "bob")
            .with_expr(text_segment([strlit("hello "), index(ident("state"), strlit("s"))]))
            .eval(|value, strings| {
                let EvalValue::String(value) = *value else { panic!() };
                let s = strings.get(value);
                assert_eq!("hello bob", s);
            });
    }
}
