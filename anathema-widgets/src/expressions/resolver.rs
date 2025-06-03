use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::ops::ControlFlow;
use std::rc::Rc;

use anathema_state::{
    register_future, CommonString, CommonVal, Number, Path, PendingValue, SharedState, StateId, States, ValueRef,
};
use anathema_strings::{HString, StrIndex, Strings, Transaction};
use anathema_templates::expressions::{Equality, Op};
use anathema_templates::{Expression, Globals};

use super::{EvalValue, ExprEvalCtx, NameThis, ValueKind};
use crate::scope::{Lookup, Scope, ScopeLookup};
use crate::values::{Collection, ValueId};
use crate::{AttributeStorage, Value, WidgetId};

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

    pub(crate) fn resolve(&mut self, expression: &'bp Expression, strings: &mut Strings<'bp>) -> ValueKind {
        match expression {
            // -----------------------------------------------------------------------------
            //   - Values -
            // -----------------------------------------------------------------------------
            &Expression::Primitive(val) => ValueKind::Common(val.into()),
            Expression::Str(s) => ValueKind::String(strings.insert_with(|tx| tx.add_slice(s))),
            Expression::TextSegments(segments) => {
                panic!("resolve into sort of things");
                // let s = strings.insert_with(|tx| {
                //     for i in inner {
                //         i.to_hoppstr(tx);
                //     }
                // });

                // EvalValue::String(s)
            }

            // Expression::Map(map) => {
            //     let inner = map
            //         .iter()
            //         .map(|(key, expr)| (key.as_str(), self.resolve(expr)))
            //         .collect();
            //     EvalValue::Map(inner)
            // }
            // Expression::Map(map) => EvalValue::Map(map),
            // Expression::List(list) if self.deferred => EvalValue::ExprList(list),
            // Expression::List(list) => {
            //     let inner = list.iter().map(|expr| self.resolve(expr, strings)).collect();
            //     EvalValue::List(inner)
            // }

            // // -----------------------------------------------------------------------------
            // //   - Conditionals -
            // // -----------------------------------------------------------------------------
            Expression::Not(expr) => match self.resolve(expr, strings) {
                ValueKind::Common(CommonVal::Bool(b)) => ValueKind::Common(CommonVal::Bool(!b)),
                _ => ValueKind::Null,
            },
            Expression::Equality(lhs, rhs, eq) => {
                let lhs = self.resolve(lhs, strings);
                let rhs = self.resolve(rhs, strings);
                match eq {
                    Equality::Eq => todo!(),
                    Equality::NotEq => todo!(),
                    Equality::And => todo!(),
                    Equality::Or => todo!(),
                    Equality::Gt => todo!(),
                    Equality::Gte => todo!(),
                    Equality::Lt => todo!(),
                    Equality::Lte => todo!(),
                }
            }

            // -----------------------------------------------------------------------------
            //   - Lookups -
            // -----------------------------------------------------------------------------
            Expression::Ident(_) | Expression::Index(_, _) => match self.lookup(expression, strings) {
                Lookup::State(state_id) => todo!(),
                Lookup::ComponentAttributes(key) => todo!(),
                Lookup::Expression(_) => todo!(),
                Lookup::Value(value_kind) => todo!(),
                Lookup::Null => todo!(),
            },

            // // -----------------------------------------------------------------------------
            // //   - Maths -
            // // -----------------------------------------------------------------------------
            // Expression::Negative(expr) => EvalValue::Negative(self.resolve(expr, strings).into()),
            // Expression::Op(lhs, rhs, op) => {
            //     let lhs = self.resolve(lhs, strings);
            //     let rhs = self.resolve(rhs, strings);
            //     EvalValue::Op(lhs.into(), rhs.into(), *op)
            // }

            // -----------------------------------------------------------------------------
            //   - Either -
            // -----------------------------------------------------------------------------
            Expression::Either(lhs, rhs) => match self.resolve(lhs, strings) {
                ValueKind::Null => self.resolve(rhs, strings),
                value => value,
            },

            // // -----------------------------------------------------------------------------
            // //   - Function call -
            // // -----------------------------------------------------------------------------
            // Expression::Call { fun: _, args: _ } => todo!(),
            _ => panic!(),
        }
    }

    fn lookup(&self, expression: &Expression, strings: &mut Strings<'bp>) -> Lookup<'bp> {
        match expression {
            Expression::Ident(ident) => match &**ident {
                "state" => self.ctx.scope.get_state(),
                "attributes" => self.ctx.scope.get_component_attributes(),
                path => {
                    let lookup = ScopeLookup::new(Path::from(path), self.subscriber);
                    match self.ctx.scope.get(lookup, &mut None, self.ctx.states) {
                        Lookup::Null => {
                            self.register_future_value = true;
                            Lookup::Null
                        }
                        Lookup::Expression(expr) => self.resolve(expr, strings),
                        Lookup::Value(value) => value,
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
