use anathema_render::Size;
use anathema_values::testing::TestState;
use anathema_values::{Context, State, Value};

use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::Result;
use crate::expressions::Expression;
use crate::layout::{Constraints, Layout};
use crate::nodes::{make_it_so, Node};
use crate::{AnyWidget, Factory, FactoryContext, LayoutNodes, Nodes, Widget, WidgetFactory};

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct TestLayoutMany;

impl Layout for TestLayoutMany {
    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        let mut size = Size::ZERO;

        let mut constraints = nodes.constraints;
        nodes.for_each(|mut node| {
            let s = node.layout(constraints)?;
            size.height += s.height;
            size.width = size.width.max(s.width);
            constraints.max_height -= size.height;
            Ok(())
        })?;

        Ok(size)
    }
}

// -----------------------------------------------------------------------------
//   - Widgets -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct TestWidget(pub Value<String>);

impl Widget for TestWidget {
    fn kind(&self) -> &'static str {
        "text"
    }

    fn layout(&mut self, _nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        match self.0.value_ref() {
            Some(s) => Ok(Size::new(s.len(), 1)),
            None => Ok(Size::ZERO),
        }
    }

    fn position<'tpl>(&mut self, _children: &mut Nodes<'_>, _ctx: PositionCtx) {}
}

struct TestWidgetFactory;

impl WidgetFactory for TestWidgetFactory {
    fn make(&self, context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = TestWidget(context.text);
        Ok(Box::new(widget))
    }
}

#[derive(Debug)]
struct TestListWidget;

impl Widget for TestListWidget {
    fn kind(&self) -> &'static str {
        "list"
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        TestLayoutMany.layout(nodes)
    }

    fn position<'tpl>(&mut self, _children: &mut Nodes<'_>, _ctx: PositionCtx) {
        todo!()
    }
}

struct TestListWidgetFactory;

impl WidgetFactory for TestListWidgetFactory {
    fn make(&self, _context: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = TestListWidget;
        Ok(Box::new(widget))
    }
}

// -----------------------------------------------------------------------------
//   - Expressions -
// -----------------------------------------------------------------------------

pub struct TestExpression<S> {
    pub state: S,
    pub expr: Box<Expression>,
    pub layout: LayoutCtx,
}

impl<S: State> TestExpression<S> {
    pub fn ctx(&self) -> Context<'_, '_> {
        Context::root(&self.state)
    }

    pub fn eval(&self) -> Result<Node<'_>> {
        self.expr.eval(&self.ctx(), 0.into())
    }
}

// -----------------------------------------------------------------------------
//   - Test node -
// -----------------------------------------------------------------------------

pub struct TestRuntime<'e> {
    pub nodes: Nodes<'e>,
    constraints: Constraints,
    state: TestState,
}

impl TestRuntime<'_> {
    pub fn layout(&mut self) -> Result<Size> {
        self.nodes.reset_cache();
        let context = Context::root(&self.state);
        let mut nodes = LayoutNodes::new(&mut self.nodes, self.constraints, &context);

        let mut size = Size::ZERO;
        nodes.for_each(|mut node| {
            let node_size = node.layout(self.constraints)?;
            size.width = size.width.max(node_size.width);
            size.height += node_size.height;
            Ok(())
        })?;

        Ok(size)
    }
}

pub fn test_runtime(exprs: &[Expression]) -> TestRuntime<'_> {
    register_test_widget();
    let nodes = make_it_so(exprs);
    TestRuntime {
        nodes,
        constraints: Constraints::new(80, 25),
        state: TestState::new(),
    }
}

// pub struct TestNodes<'e> {
//     pub nodes: Nodes<'e>,
//     state: TestState,
// }

// impl<'e> TestNodes<'e> {
//     pub fn new(exprs: &'e [Expression]) -> Self {
//         register_test_widget();
//         let nodes = Nodes::new(exprs, 0.into());
//         Self {
//             nodes,
//             state: TestState::new(),
//         }
//     }

//     pub fn layout(&mut self) -> Result<Size> {
//         let context = Context::root(&self.state);
//         let mut layout_nodes = LayoutNodes::new(
//             &mut self.nodes,
//             Constraints::new(120, 40),
//             Padding::ZERO,
//             &context,
//         );
//         TestLayoutMany.layout(&mut layout_nodes)
//     }
// }

pub(crate) fn register_test_widget() {
    let _ = Factory::register("test", TestWidgetFactory);
    let _ = Factory::register("list", TestListWidgetFactory);
}
