use std::rc::Rc;
use std::str::FromStr;

use anathema_render::Size;
use anathema_values::testing::TestState;
use anathema_values::{Context, NodeId, Path, Scope, State, ValueExpr, ValueRef};

use super::nodes::Node;
use super::{ControlFlow, If, Else};
use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::Result;
use crate::generator::expressions::{Expression, Loop, SingleNode};
use crate::layout::{Constraints, Layout, Layouts};
use crate::{
    AnyWidget, Attributes, Factory, Nodes, Padding, Widget, WidgetContainer, WidgetFactory, FactoryContext,
};

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct TestLayoutMany;

impl Layout for TestLayoutMany {
    fn layout(
        &mut self,
        children: &mut Nodes,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        let mut size = Size::ZERO;

        children.for_each(data, layout, |widget, children, ctx| {
            let s = widget.layout(children, layout.constraints, ctx)?;
            size.height += s.height;
            size.width = size.width.max(s.width);
            Ok(())
        });

        Ok(size)
    }
}

// -----------------------------------------------------------------------------
//   - Widgets -
// -----------------------------------------------------------------------------

struct TestWidget(Value<String>);

impl Widget for TestWidget {
    fn kind(&self) -> &'static str {
        "text"
    }

    fn layout(
        &mut self,
        children: &mut Nodes<'_>,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        match self.0.len() {
            0 => Ok(Size::ZERO),
            width => Ok(Size::new(width, 1)),
        }
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx) {}
}

struct TestWidgetFactory;

impl WidgetFactory for TestWidgetFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let text = context.text();
        let widget = TestWidget(text);
        Ok(Box::new(widget))
    }
}

struct TestListWidget;

impl Widget for TestListWidget {
    fn kind(&self) -> &'static str {
        "list"
    }

    fn layout(
        &mut self,
        children: &mut Nodes<'_>,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        let mut layout = Layouts::new(TestLayoutMany, layout);
        layout.layout(children, data)
    }

    fn position<'tpl>(&mut self, children: &mut Nodes, ctx: PositionCtx) {
        todo!()
    }
}

struct TestListWidgetFactory;

impl WidgetFactory for TestListWidgetFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = TestListWidget;
        Ok(Box::new(widget))
    }
}

// -----------------------------------------------------------------------------
//   - Expressions -
// -----------------------------------------------------------------------------

pub struct TestExpression<'a, S> {
    pub state: S,
    pub scope: Scope<'a>,
    pub expr: Box<Expression>,
    pub layout: LayoutCtx,
}

impl<'a, S: State> TestExpression<'a, S> {
    pub fn ctx(&self) -> Context<'_, '_> {
        let ctx = Context::new(&self.state, &self.scope);
        ctx
    }

    pub fn eval(&'a self) -> Result<Node<'a>> {
        self.expr.eval(&self.ctx(), 0.into())
    }
}

// -----------------------------------------------------------------------------
//   - Test node -
// -----------------------------------------------------------------------------
pub struct TestNodes<'e> {
    pub nodes: Nodes<'e>,
    scope: Scope<'e>,
    state: TestState,
}

impl<'e> TestNodes<'e> {
    pub fn new(exprs: &'e [Expression]) -> Self {
        register_test_widget();
        let nodes = Nodes::new(exprs, 0.into());
        Self {
            nodes,
            scope: Scope::new(None),
            state: TestState::new(),
        }
    }

    pub fn layout(&mut self) -> Result<Size> {
        let context = Context::new(&self.state, &self.scope);
        let constraints = Constraints::new(120, 40);
        let layout = LayoutCtx::new(constraints, Padding::ZERO);

        TestLayoutMany.layout(&mut self.nodes, &layout, &context)
    }
}

pub(crate) fn register_test_widget() {
    Factory::register("test", TestWidgetFactory);
    Factory::register("list", TestListWidgetFactory);
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
    Expression::Loop(Loop {
        body: body.into().into(),
        binding: binding.into(),
        collection: *collection,
    })
}

pub(crate) fn if_expression(
    if_true: (ValueExpr, Vec<Expression>),
    elses: Vec<(Option<ValueExpr>, Vec<Expression>)>,
    //     binding: impl Into<Path>,
    //     collection: Box<ValueExpr>,
    //     body: impl Into<Vec<Expression>>,
) -> Expression {
    Expression::ControlFlow(ControlFlow {
        if_expr: If {
            cond: if_true.0,
            body: if_true.1,
        },
        elses: elses
            .into_iter()
            .map(|(cond, body)| Else { cond, body })
            .collect(),
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
