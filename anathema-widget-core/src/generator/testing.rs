use std::rc::Rc;
use std::str::FromStr;

use anathema_values::{Context, Path, Scope, ScopeValue, State, ValueExpr};

use crate::error::Result;
use crate::generator::expressions::{Expression, Loop, SingleNode};
use crate::{Attributes, Factory, Widget, WidgetContainer, WidgetFactory};

use super::nodes::Node;

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

struct TestWidget;

impl Widget for TestWidget {
    fn kind(&self) -> &'static str {
        "test"
    }

    fn layout(
        &mut self,
        children: &mut crate::Nodes,
        ctx: &mut crate::contexts::LayoutCtx,
        data: Context<'_, '_>,
    ) -> crate::error::Result<anathema_render::Size> {
        todo!()
    }

    fn position<'tpl>(&mut self, children: &mut crate::Nodes, ctx: crate::contexts::PositionCtx) {
        todo!()
    }
}

struct TestWidgetFactory;

impl WidgetFactory for TestWidgetFactory {
    fn make(
        &self,
        data: Context<'_, '_>,
        attributes: &Attributes,
        text: Option<&ValueExpr>,
        noden_id: &anathema_values::NodeId,
    ) -> crate::error::Result<Box<dyn crate::AnyWidget>> {
        let widget = TestWidget;
        Ok(Box::new(widget))
    }
}

pub struct TestExpression<'a, S> {
    pub state: S,
    pub scope: Scope<'a>,
    pub expr: Box<Expression>,
}

impl<'a, S: State> TestExpression<'a, S> {
    pub fn eval(&'a self) -> Result<Node<'a>> {
        self.expr.eval(&self.state, &self.scope, 0.into())
    }
}

pub(crate) fn register_test_widget() {
    Factory::register("test", TestWidgetFactory);
}

pub(crate) fn expression(
    ident: impl Into<String>,
    text: impl Into<Option<ValueExpr>>,
    attributes: impl Into<Attributes>,
    children: impl Into<Vec<Expression>>,
) -> Expression {
    let children = children.into();
    Expression::Node(SingleNode {
        ident: ident.into(),
        text: text.into(),
        attributes: attributes.into(),
        children: children.into(),
    })
}

pub(crate) fn for_expression(
    binding: impl Into<Path>,
    collection: Box<ValueExpr>,
    body: impl Into<Vec<Expression>>,
) -> Expression {
    // let collection = collection.map(Into::into);
    // let binding = binding.into();
    Expression::Loop(Loop {
        body: body.into().into(),
        binding: binding.into(),
        collection: *collection,
    })
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
