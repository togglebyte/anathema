use anathema_render::{Screen, ScreenPos, Size};
use anathema_values::testing::TestState;
use anathema_values::Context;

pub use self::expressions::expression;
use super::WidgetContainer;
use crate::contexts::PaintCtx;
use crate::expressions::Expression;
use crate::layout::Constraints;
use crate::nodes::{NodeKind, Single};
use crate::{Node, Nodes, Pos};

pub mod expressions;
pub mod nodes;

impl<'e> Node<'e> {
    pub(crate) fn single(&mut self) -> (&mut WidgetContainer<'e>, &mut Nodes<'e>) {
        match &mut self.kind {
            NodeKind::Single(Single {
                widget, children, ..
            }) => (widget, children),
            _ => panic!(),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Here be (hacky) dragons -
//   What you are about to see here might cause you to scream and run away.
//
//   This exists to make tests readable.
//   Before you judge me too hard, know that I am a loving father,
//   I care for two bunnies that roam free in my house (eating the wiring),
//   I give to charity when I can, and I always try to help others
//   as much as possible.
//
//   No thought has gone into making this code nice, readable, or efficient.
//   There is but one purpose of this code: to easily write readable tests.
//
//   Knowing this you are now free to judge me...
// -----------------------------------------------------------------------------

pub struct FakeTerm {
    screen: Screen,
    size: Size,
    rows: Vec<String>,
}

impl FakeTerm {
    pub fn from_str(s: &str) -> Self {
        let mut size = Size::ZERO;

        let lines = s.lines().map(|l| l.trim()).filter(|l| !l.is_empty());
        let mut expected = vec![];
        let mut collect = false;

        for line in lines {
            if line.contains("] Fake term [") {
                size.width = line.chars().count() - 2;
                collect = true;
                continue;
            }
            if line.starts_with('║') && line.ends_with('║') {
                size.height += 1;
                if collect {
                    let mut l = line.chars().skip(1).collect::<String>();
                    l.pop();
                    expected.push(l);
                }
            }
        }

        Self::new(size, expected)
    }

    pub fn new(size: Size, rows: Vec<String>) -> Self {
        let screen = Screen::new(size);
        Self { screen, size, rows }
    }

    fn rendered_output(&self) -> String {
        let mut output_rows = vec![];
        for y in 0..self.size.height {
            let mut row = String::new();
            for x in 0..self.size.width {
                let c = self
                    .screen
                    .get(ScreenPos::new(x as u16, y as u16))
                    .map(|(c, _)| c)
                    .unwrap_or('E');
                row.push(c);
            }
            output_rows.push(row);
        }

        output_rows.join("\n")
    }
}

pub fn test_widget(expr: Expression, expected: FakeTerm) {
    let state = TestState::new();
    let context = Context::root(&state);
    let mut node = expr.eval(&context, 0.into()).unwrap();
    let (widget, nodes) = node.single();

    test_widget_container(widget, nodes, &context, expected)
}

pub fn test_widget_container<'e>(
    widget: &mut WidgetContainer<'e>,
    children: &mut Nodes<'e>,
    context: &Context<'_, 'e>,
    mut expected: FakeTerm,
) {
    // Layout
    let constraints = Constraints::new(Some(expected.size.width), Some(expected.size.height));
    widget.layout(children, constraints, context).unwrap();

    // Position
    widget.position(children, Pos::ZERO);

    // Paint
    let ctx = PaintCtx::new(&mut expected.screen, None);
    widget.paint(children, ctx);

    let expected_rows = expected.rows.iter();
    for (y, row) in expected_rows.enumerate() {
        for (x, c) in row.chars().enumerate() {
            let pos = ScreenPos::new(x as u16, y as u16);
            match expected.screen.get(pos).map(|(c, _)| c) {
                Some(buffer_char) => assert_eq!(
                    c,
                    buffer_char,
                    "in fake term \"{c}\", in buffer \"{buffer_char}\", pos: {pos:?}\n{}",
                    expected.rendered_output()
                ),
                None if c == ' ' => continue,
                None => {
                    panic!(
                        "expected {c}, found nothing\noutput: {}",
                        expected.rendered_output()
                    )
                }
            }
        }
    }
}
