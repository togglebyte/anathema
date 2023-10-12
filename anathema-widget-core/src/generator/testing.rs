use std::rc::Rc;
use std::str::FromStr;

use anathema_render::Size;
use anathema_values::testing::TestState;
use anathema_values::{Context, Path, Scope, ScopeValue, State, ValueExpr};

use super::nodes::builder::NodeBuilder;
use super::nodes::Node;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::expressions::{Expression, Loop, SingleNode};
use crate::layout::{Constraints, Layouts, Layout};
use crate::{Attributes, Factory, Nodes, Padding, Widget, WidgetContainer, WidgetFactory};

struct TestLayout {
}

impl Layout for TestLayout {
    fn layout(
        &mut self,
        layout: &mut LayoutCtx,
        children: &mut Nodes,
        data: &Context<'_, '_>,
        size: &mut Size,
    ) -> Result<Size> {
        let mut builder = NodeBuilder::new(constraints, size);
        loop {
            children.next(&mut builder, data)
        }
    }
}

struct TestWidget;

struct TestLayout;

impl Layout for TestLayout {
    fn layout(
        &mut self,
        layout: &mut LayoutCtx,
        children: &mut Nodes,
        data: &Context<'_, '_>,
    ) -> Result<anathema_render::Size> {
        Ok(Size::new(5, 5))
    }
}

impl Widget for TestWidget {
    fn kind(&self) -> &'static str {
        "test"
    }

    fn layout(
        &mut self,
        children: &mut crate::Nodes<'_>,
        ctx: &mut crate::contexts::LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        let mut layout = Layouts::new(TestLayout, layout);
        let size = layout.layout(children, data)?;
        if size == Size::ZERO {
            Ok(Size::ZERO)
        } else {
            Ok(layout.expand_horz().expand_vert().size())
        }
    }

    fn position<'tpl>(&mut self, children: &mut crate::Nodes, ctx: crate::contexts::PositionCtx) {
        todo!()
    }
}

struct TestWidgetFactory;

impl WidgetFactory for TestWidgetFactory {
    fn make(
        &self,
        data: &Context<'_, '_>,
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

    pub fn next(&mut self) -> Option<Result<()>> {
        let context = Context::new(&self.state, &self.scope);
        let mut visitor = NodeBuilder {
            layout: LayoutCtx::new(Constraints::new(120, 40), Padding::ZERO),
            context,
        };
        match self.nodes.next(&mut visitor, &context)? {
            Ok(()) => {
                self.nodes.advance();
                Some(Ok(()))
            }
            Err(e) => panic!("{e}"),
        }
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
