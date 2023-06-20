

use anathema_render::{Screen, ScreenPos, Size};

use super::WidgetContainer;
use crate::gen::store::Store;
use crate::template::Template;
use crate::{Constraints, DataCtx, Lookup, PaintCtx, Pos, Widget};

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
}

pub fn test_widget(widget: impl Widget + 'static, children: &[Template], expected: FakeTerm) {
    let widget = WidgetContainer::new(Box::new(widget), children);
    test_widget_container(widget, expected)
}

pub fn test_widget_container(mut widget: WidgetContainer<'_>, mut expected: FakeTerm) {
    // Layout
    let constraints = Constraints::new(Some(expected.size.width), Some(expected.size.height));
    let lookup = Lookup::default();
    let data = DataCtx::default();
    let store = Store::new(&data);
    widget.layout(constraints, &store, &lookup).unwrap();

    // Position
    widget.position(Pos::ZERO);

    // Paint
    let ctx = PaintCtx::new(&mut expected.screen, None);
    widget.paint(ctx);

    let expected_rows = expected.rows.iter();
    for (y, row) in expected_rows.enumerate() {
        for (x, c) in row.chars().enumerate() {
            let pos = ScreenPos::new(x as u16, y as u16);
            match expected.screen.get(pos).map(|(c, _)| c) {
                Some(buffer_char) => assert_eq!(
                    c, buffer_char,
                    "in fake term \"{c}\", in buffer \"{buffer_char}\", pos: {pos:?}"
                ),
                None if c == ' ' => continue,
                None => panic!("expected {c}, found nothing"),
            }
        }
    }
}
