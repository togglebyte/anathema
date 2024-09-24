use anathema_state::{CommonVal, Path, States};
use anathema_templates::{Expression, Globals};

use crate::components::ComponentAttributeCollection;
use crate::expressions::EvalValue;
use crate::scope::ScopeLookup;
use crate::values::ValueId;
use crate::{Scope, Value};

pub struct Resolver2<'scope, 'bp> {
    globals: &'bp Globals,
    scope_level: usize,
    subscriber: ValueId,
    scope: &'scope Scope<'bp>,
    states: &'scope States,
    component_attributes: &'scope ComponentAttributeCollection<'bp>,
}

impl<'scope, 'bp> Resolver2<'scope, 'bp> {
    pub(crate) fn new(
        scope_level: usize,
        scope: &'scope Scope<'bp>,
        states: &'scope States,
        component_attributes: &'scope ComponentAttributeCollection<'bp>,
        globals: &'bp Globals,
        subscriber: ValueId,
    ) -> Self {
        Self {
            scope,
            states,
            component_attributes,
            globals,
            scope_level,
            subscriber,
        }
    }

    pub(crate) fn root(
        scope: &'scope Scope<'bp>,
        states: &'scope States,
        component_attributes: &'scope ComponentAttributeCollection<'bp>,
        globals: &'bp Globals,
        subscriber: ValueId,
    ) -> Self {
        Self::new(0, scope, states, component_attributes, globals, subscriber)
    }

    pub fn resolve(&mut self, expression: &'bp Expression) -> EvalValue<'bp> {
        match expression {
            // -----------------------------------------------------------------------------
            //   - Values -
            // -----------------------------------------------------------------------------
            &Expression::Primitive(val) => EvalValue::Static(val.into()),
            Expression::Str(s) => EvalValue::Static(CommonVal::Str(s)),
            Expression::Map(map) => {
                let inner = map
                    .iter()
                    .map(|(key, expr)| (key.clone(), self.resolve(expr)))
                    .collect();
                EvalValue::ExprMap(inner)
            }
            Expression::List(list) => {
                let inner = list.iter().map(|expr| self.resolve(expr)).collect();
                EvalValue::ExprList(inner)
            }

            // -----------------------------------------------------------------------------
            //   - Conditionals -
            // -----------------------------------------------------------------------------
            Expression::Not(expr) => EvalValue::Not(self.resolve(expr).into()),
            Expression::Equality(lhs, rhs, eq) => {
                EvalValue::Equality(self.resolve(lhs).into(), self.resolve(rhs).into(), *eq)
            }

            // -----------------------------------------------------------------------------
            //   - Lookups -
            // -----------------------------------------------------------------------------
            Expression::Ident(_) | Expression::Index(_, _) => self.lookup(expression),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            Expression::Negative(expr) => EvalValue::Negative(self.resolve(expr).into()),
            Expression::Op(lhs, rhs, op) => {
                let lhs = self.resolve(lhs);
                let rhs = self.resolve(rhs);
                EvalValue::Op(lhs.into(), rhs.into(), *op)
            }

            // -----------------------------------------------------------------------------
            //   - Either -
            // -----------------------------------------------------------------------------
            Expression::Either(lhs, rhs) => match self.resolve(lhs) {
                EvalValue::Empty => self.resolve(rhs),
                value => value,
            },

            // -----------------------------------------------------------------------------
            //   - Function call -
            // -----------------------------------------------------------------------------
            Expression::Call { fun: _, args: _ } => todo!(),
        }
    }

    fn lookup(&mut self, expression: &'bp Expression) -> EvalValue<'bp> {
        match expression {
            Expression::Ident(ident) => match &**ident {
                path @ "state" => self.scope.get_state(self.states),
                path @ "attributes" => self.scope.get_component_attributes(),
                path => {
                    let lookup = ScopeLookup::new(Path::from(path), self.subscriber);
                    match self.scope.get(lookup, &mut None, self.states) {
                        Some(val) => val,
                        None => match self.globals.get(path) {
                            Some(value) => self.resolve(value),
                            None => EvalValue::Empty,
                        }
                    }
                }
            },
            Expression::Index(lhs, rhs) => {
                let value = self.resolve(lhs);

                // The RHS is always the index / ident.
                // Note that this might still be an op, e.g a + 1
                // So the expression has to be evaluated before it can be used as an index.
                //
                // Once evaluated it should be either a string or a number
                let index = match &**rhs {
                    Expression::Str(ident) => Path::from(&**ident),
                    expr => {
                        let index = self.resolve(expr);
                        let index = index.load_number().unwrap().as_int() as usize;
                        Path::from(index)
                    }
                };

                let val = match value.get(index, self.subscriber, self.states, self.component_attributes) {
                    Some(val) => val,
                    None => EvalValue::Empty,
                };
                val
            }
            _ => unreachable!("lookup only handles ident and index"),
        }
    }
}

// pub(crate) fn eval2<'bp>(
//     expr: &'bp Expression,
//     globals: &'bp Globals,
//     scope: &Scope<'bp>,
//     states: &States,
//     value_id: impl Into<ValueId>,
// ) -> Value<'bp, EvalValue<'bp>> {
//     let value_id = value_id.into();
//     let value = Resolver2::root(scope, states, globals, value_id).resolve(expr);
//     Value::new(value, Some(expr))
// }

#[cfg(test)]
mod test {
    use anathema_state::{List, Map, Value};
    use anathema_templates::expressions::{ident, index, less_than_equal, num, strlit};

    use super::*;
    use crate::testing::ScopedTest;

    #[test]
    fn attribute_lookup() {
        let mut t = ScopedTest::new()
            .with_value("a", 123u32)
            .with_expr(index(ident("state"), ident("a")));

        t.eval(|value| {
            let val = value.load::<u32>().unwrap();
            assert_eq!(val, 123);
        });
    }

    #[test]
    fn index_lookup_on_lists_of_maps2() {
        let mut map = Map::empty();
        map.insert("val", 1u32);

        let mut lists = List::<Value<Map<u32>>>::empty();
        lists.push_back(map);

        ScopedTest::new()
            .with_value("list", lists)
            .with_expr(index(
                index(index(ident("state"), ident("list")), num(0)),
                strlit("val"),
            ))
            .eval(|value| {
                let val = value.load::<u32>().unwrap();
                assert_eq!(val, 1);
            });
    }

    #[test]
    fn index_lookup_on_lists_of_lists2() {
        let mut numbers = List::empty();
        numbers.push_back(123u32);

        let mut lists = List::<List<_>>::empty();
        lists.push_back(numbers);

        ScopedTest::new()
            .with_value("a", lists)
            .with_expr(index(index(index(ident("state"), ident("a")), num(0)), num(0)))
            .eval(|value| {
                let val = value.load::<u32>().unwrap();
                assert_eq!(val, 123);
            });
    }

    #[test]
    fn lesser_equal_expr2() {
        ScopedTest::new()
            .with_value("a", 2)
            .with_value("b", 2)
            .with_expr(less_than_equal(
                index(ident("state"), ident("a")),
                index(ident("state"), ident("b")),
            ))
            .eval(|value| {
                let b = value.load::<bool>().unwrap();
                assert!(b);
            });
    }
}
