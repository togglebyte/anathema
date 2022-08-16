use crate::display::{Screen, ScreenPos, Size};

use super::layout::Constraints;
use super::{NodeId, PaintCtx, Pos, Widget, WidgetContainer};

pub fn test_widget(widget: impl Widget, expected: &str) -> WidgetContainer {
    let container = widget.into_container(NodeId::auto());
    test_widget_container(container, expected)
}

pub fn test_widget_container(mut widget: WidgetContainer, expected: &str) -> WidgetContainer {
    let lines = expected
        .lines()
        .filter_map(|s| {
            let s = s.trim();
            match s.len() {
                0 => None,
                _ => Some(s),
            }
        })
        .collect::<Vec<_>>();
    let expected = lines.join("\n");

    // The size of the screen
    let width = lines.iter().map(|s| s.chars().count()).max().unwrap();
    let height = lines.len();
    let size = Size::new(width, height);

    // Screen setup
    let mut screen = Screen::new(&mut vec![], size).unwrap();
    let paint_ctx = PaintCtx::new(&mut screen, None);

    // Layout and paint
    let mut constraints = Constraints::new(size.width, size.height);
    constraints.make_width_tight(constraints.max_width);
    constraints.make_height_tight(constraints.max_height);
    let _size = widget.layout(constraints, false);

    widget.position(Pos::ZERO);
    widget.paint(paint_ctx);

    // Build up the actual value, this is helpful if the test
    // fails, as it;s possible to display the actual outcome
    let mut actual = String::new();
    for (y, line) in lines.iter().enumerate() {
        for (x, _) in line.chars().enumerate() {
            let pos = ScreenPos::new(x as u16, y as u16);
            if let Some((buffer_value, _)) = screen.get(pos) {
                actual.push(buffer_value);
            } else {
                actual.push(' ');
            }
        }
        actual.push('\n');
    }

    for (y, line) in lines.into_iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let pos = ScreenPos::new(x as u16, y as u16);
            let buffer_value = screen.get(pos);

            match buffer_value {
                Some((buf_char, _)) => assert_eq!(buf_char, c, "\nexpected:\n{}\nfound:\n{}", expected, actual),
                None => assert_eq!(c, ' ', "expected:\n{}\nfound:\n{}", expected, actual),
            }
        }
    }

    widget
}
