use std::fmt::Display;
use std::rc::Rc;

use crate::{Context, NodeId, Num, Owned, Path, Scope, ScopeValue, State, Value, ValueRef};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    Value(Value),

    Not(Box<ValueExpr>),
    Negative(Box<ValueExpr>),

    Ident(Rc<str>),
    List(Vec<ValueExpr>),
    Key(Box<ValueExpr>),
    Index(Box<ValueExpr>, Box<ValueExpr>),
    Add(Box<ValueExpr>, Box<ValueExpr>),
    Sub(Box<ValueExpr>, Box<ValueExpr>),
    Div(Box<ValueExpr>, Box<ValueExpr>),
    Mul(Box<ValueExpr>, Box<ValueExpr>),
    And(Box<ValueExpr>, Box<ValueExpr>),
    Mod(Box<ValueExpr>, Box<ValueExpr>),
    Or(Box<ValueExpr>, Box<ValueExpr>),

    Dot(Box<ValueExpr>, Box<ValueExpr>),

    Invalid,
}

impl Display for ValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(val) => write!(f, "{val}"),
            Self::Ident(s) => write!(f, "{s}"),
            Self::Key(n) => write!(f, "{n}"),
            Self::Index(lhs, idx) => write!(f, "{lhs}[{idx}]"),
            Self::Dot(lhs, rhs) => write!(f, "{lhs}.{rhs}"),
            Self::Not(expr) => write!(f, "!{expr}"),
            Self::Negative(expr) => write!(f, "-{expr}"),
            Self::Add(lhs, rhs) => write!(f, "{lhs} + {rhs}"),
            Self::Sub(lhs, rhs) => write!(f, "{lhs} - {rhs}"),
            Self::Mul(lhs, rhs) => write!(f, "{lhs} * {rhs}"),
            Self::Div(lhs, rhs) => write!(f, "{lhs} / {rhs}"),
            Self::Mod(lhs, rhs) => write!(f, "{lhs} % {rhs}"),
            _ => panic!("{self:#?}"),
        }
    }
}

impl<T> From<T> for ValueExpr
where
    T: Into<Value>,
{
    fn from(val: T) -> Self {
        Self::Value(val.into())
    }
}

// ItemState {
//     name: Value<String>,
//     age: Value<usize>,
// }
//
// RootState {
//    collection: List<ItemState>,
//    root_num: Value<u32>,
// }
//
// Template
// --------
//
// // scope value `item` from collection, subscribe for-loop to `collection`
// for item in collection
//     ValueExpr::Add(
//         ValueExpr::Val(Dyn("item", "age")), ValueExpr::Sub(
//             Dyn("root_num"),
//             Static(1)
//         )
//     )
//     text "{{ item.age + root_num - 1 }}"

// y = [
//    [1, 2, 3],
//    [4, some_ident, 5],
// ]
//
// sausages = [1, 2, 3, 4, 5, 6]
// some_ident = 1
//
// for x in y // x = [4, some_ident, 5]
//     for a in x
//         text sausages[a]
//
// a -> some_ident -> 1 // this means we are storing `a` as an expression inside a `Scope`
// sausages[1] -> 2

impl ValueExpr {
    pub fn eval<'val, T: 'val>(&'val self, context: &Context<'_, 'val>, node_id: Option<&NodeId>) -> Option<&'val T>
    where
        for<'b> &'b T: TryFrom<&'b Value>,
        for<'b> &'b T: TryFrom<ValueRef<'b>>,
    {
        match self {
            Self::Value(value) => value.try_into().ok(),
            expr @ (Self::Dot(..) | Self::Ident(_)) => {
                let path = eval_path(expr, context, node_id)?;
                context.get::<T>(&path, node_id)
            }
            _ => panic!(),
        }
    }
}

fn eval_path(
    expr: &ValueExpr,
    context: &Context<'_, '_>,
    node_id: Option<&NodeId>,
) -> Option<Path> {
    let path = match expr {
        ValueExpr::Ident(key) => Path::Key(key.to_string()),
        ValueExpr::Dot(lhs, rhs) => Path::Composite(
            eval_path(lhs, context, node_id)?.into(),
            eval_path(rhs, context, node_id)?.into(),
        ),
        ValueExpr::Index(lhs, index) => {
            let index = *index.eval::<u64>(context, node_id)?;
            let collection = eval_path(lhs, context, node_id)?;
            collection.compose(Path::Index(index as usize))
        }
        _ => return None,
    };

    Some(path)
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::ops::Deref;

    use super::*;
    use crate::{List, Scope, State, StateValue};

    struct Inner {
        name: StateValue<String>,
        names: List<String>,
    }

    impl State for Inner {
        fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Cow<'_, Value>> {
            match key {
                Path::Key(key) if key == "name" => {
                    let num: &str = &*self.name;
                    return Some(Cow::Owned(Value::Str(num.into())));
                }
                Path::Composite(lhs, rhs) => {
                    let lhs: &Path = &*lhs;
                    match lhs {
                        Path::Key(key) if key == "names" => return self.names.lookup(rhs, node_id),
                        _ => {}
                    }
                }
                _ => {}
            }

            None
        }

        fn get_collection(
            &self,
            key: &Path,
            node_id: Option<&NodeId>,
        ) -> Option<crate::Collection> {
            None
        }
    }

    struct TheState {
        counter: StateValue<usize>,
        some_ident: StateValue<String>,
        inner: Inner,
    }

    impl State for TheState {
        fn get<T>(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Cow<'_, Value>> {
            match key {
                Path::Key(key) if key == "counter" => {
                    let num: usize = *self.counter;
                    Some(Cow::Owned(Value::Num(num.into())))
                }
                Path::Key(key) if key == "some_ident" => {
                    let s: &str = &*self.some_ident;
                    Some(Cow::Owned(Value::Str(s.into())))
                }
                Path::Composite(lhs, rhs) => {
                    let lhs: &Path = &*lhs;
                    if let Path::Key(key) = lhs {
                        if key == "inner" {
                            return self.inner.get(rhs, node_id);
                        }
                    }
                    None
                }
                _ => None,
            }
        }

        fn get_collection(
            &self,
            key: &Path,
            node_id: Option<&NodeId>,
        ) -> Option<crate::Collection> {
            None
        }
    }

    #[test]
    fn resolve_something() {
        let mut scope = Scope::new(None);
        scope.scope(
            "a".into(),
            Cow::Owned(ScopeValue::Expr(ValueExpr::Ident("some_ident".into()))),
        );

        scope.scope(
            "some_ident".into(),
            Cow::Owned(ScopeValue::Static(Value::Num(Num::Unsigned(1)))),
        );

        // for x in y // x = [4, some_ident, 5]
        //     for a in x
        //         text sausages[a]

        let mut state = TheState {
            counter: StateValue::new(123),
            some_ident: StateValue::new("Hello this is amazing!".to_string()),
            inner: Inner {
                name: StateValue::new("Fin the human".to_string()),
                names: List::new(vec![
                    StateValue::new("First".to_string()),
                    StateValue::new("Second".into()),
                ]),
            },
        };

        // let value_expr = ValueExpr::Ident("counter".into());
        let node_id = NodeId::new(123);

        // let val = value_expr
        //     .eval(&mut Context::new(&mut state, &mut scope), Some(&node_id))
        //     .unwrap();

        // panic!("{val:#?}");
        // assert_eq!(val, "123");

        // inner.name
        // let value_expr = ValueExpr::Dot(
        //     ValueExpr::Ident("inner".into()).into(),
        //     ValueExpr::Index("name".into()).into(),
        // );

        let value_expr = ValueExpr::Dot(
            ValueExpr::Ident("inner".into()).into(),
            ValueExpr::Index(
                ValueExpr::Ident("names".into()).into(),
                ValueExpr::Ident("a".into()).into(),
            )
            .into(),
        );

        let val = value_expr
            .eval(&mut Context::new(&mut state, &mut scope), Some(&node_id))
            .unwrap();

        panic!("If you can read this things are pretty good: {val:#?}");
    }
}
