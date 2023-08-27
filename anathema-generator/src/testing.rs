use std::rc::Rc;
use std::str::FromStr;

use anathema_values::{Path, State};

use crate::expressions::Expression;
use crate::{Attributes, IntoWidget, Value};

// // -----------------------------------------------------------------------------
// //   - Helper impls -
// // -----------------------------------------------------------------------------
// impl<T: Clone> Into<ExpressionValues<T>> for Vec<(String, T)> {
//     fn into(self) -> ExpressionValues<T> {
//         let mut values = ExpressionValues::empty();
//         for (k, v) in self {
//             values.set(k, ExpressionValue::Static(v.into()));
//         }
//         values
//     }
// }

// impl<T: Clone, const N: usize> Into<ExpressionValues<T>> for [(String, T); N] {
//     fn into(self) -> ExpressionValues<T> {
//         let mut values = ExpressionValues::empty();
//         for (k, v) in self {
//             values.set(k, ExpressionValue::Static(v.into()));
//         }
//         values
//     }
// }

// impl<T: Clone> Into<ExpressionValues<T>> for () {
//     fn into(self) -> ExpressionValues<T> {
//         ExpressionValues::empty()
//     }
// }

// impl<T> From<T> for ExpressionValue<T> {
//     fn from(value: T) -> Self {
//         ExpressionValue::Static(value.into())
//     }
// }

// impl<T, const N: usize> From<[T; N]> for ExpressionValue<T> {
//     fn from(values: [T; N]) -> Self {
//         ExpressionValue::List(values.map(Into::into).into())
//     }
// }

// pub(crate) struct Expressions<T: FromContext>(Vec<Expression<T>>);

// impl<T: FromContext> Into<Expressions<T>> for Expression<T> {
//     fn into(self) -> Expressions<T> {
//         Expressions(vec![self])
//     }
// }

// impl<T: FromContext, E> Into<Expressions<T>> for Vec<E>
// where
//     E: Into<Expression<T>>,
// {
//     fn into(self) -> Expressions<T> {
//         let mut output = vec![];
//         for expr in self {
//             output.push(expr.into())
//         }
//         Expressions(output)
//     }
// }

// impl<T: FromContext, const N: usize, E> Into<Expressions<T>> for [E; N]
// where
//     E: Into<Expression<T>>,
// {
//     fn into(self) -> Expressions<T> {
//         let mut output = vec![];
//         for expr in self {
//             output.push(expr.into())
//         }
//         Expressions(output)
//     }
// }

// impl<T: FromContext> Into<Expressions<T>> for () {
//     fn into(self) -> Expressions<T> {
//         Expressions(vec![])
//     }
// }

// impl<T: Truthy> From<T> for ControlFlowExpr<T> {
//     fn from(value: T) -> Self {
//         ControlFlowExpr::If(ExpressionValue::Static(value.into()))
//     }
// }

// impl<T> Into<ControlFlowExpr<T>> for Option<T> {
//     fn into(self) -> ControlFlowExpr<T> {
//         ControlFlowExpr::Else(self.map(|val| ExpressionValue::Static(val.into())))
//     }
// }

// // -----------------------------------------------------------------------------
// //   - Listener -
// // -----------------------------------------------------------------------------
// pub(crate) struct Listener;

// impl Listen for Listener {
//     type Key = NodeId;
//     type Value = u32;

//     fn subscribe(value: ValueRef<Container<Self::Value>>, key: Self::Key) {}
// }

impl From<()> for Attributes {
    fn from((): ()) -> Self {
        Attributes::empty()
    }
}

impl<T: std::fmt::Display> From<T> for Value {
    fn from(s: T) -> Self {
        let s = s.to_string();
        Self::Static(s.into())
    }
}

fn real() {
    let v: Vec<()> = [].into();
}

// -----------------------------------------------------------------------------
//   - Test widget -
// -----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Widget {
    pub ident: Rc<str>,
}

impl IntoWidget for Widget {
    type Err = ();
    type Meta = str;
    type State = ();

    fn create_widget(
        meta: &Rc<Self::Meta>,
        state: &Self::State,
        attributes: &Attributes,
    ) -> Result<Self, Self::Err> {
        Ok(Widget { ident: meta.clone() })
    }

    fn layout(&mut self, children: &mut crate::Nodes<Self>) {
    }
}

pub(crate) fn expression(
    context: impl Into<Rc<str>>,
    attributes: impl Into<Attributes>,
    children: impl Into<Vec<Expression<Widget>>>,
) -> Expression<Widget> {
    let children = children.into();
    Expression::Node {
        context: context.into(),
        attributes: attributes.into(),
        children: children.into(),
    }
}

pub(crate) fn for_expression<const N: usize>(
    binding: impl Into<Path>,
    collection: [impl Into<Value>; N],
    body: impl Into<Vec<Expression<Widget>>>,
) -> Expression<Widget> {
    let collection = collection.map(Into::into);
    let binding = binding.into();
    Expression::Loop {
        body: body.into().into(),
        loop_repr: Rc::new(Loop::new(binding, collection.into())),
    }
}

// pub(crate) fn controlflow<E>(
//     flows: impl Into<Vec<(ControlFlowExpr<<Widget as FromContext>::Value>, E)>>,
// ) -> Expression<Widget>
// where
//     E: Into<Expressions<Widget>>,
// {
//     let flows = flows
//         .into()
//         .into_iter()
//         .map(|(val, exprs)| (val, exprs.into().0.into()))
//         .collect();

//     Expression::ControlFlow(flows)
// }
