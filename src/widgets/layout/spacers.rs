use super::Constraints;
use crate::display::Size;
use crate::widgets::{Direction, Spacer, WidgetContainer};

pub fn layout(
    spacers: &mut [WidgetContainer],
    mut constraints: Constraints,
    force_layout: bool,
    direction: Direction,
) -> Size {
    let mut size = Size::ZERO;
    let count = spacers.iter_mut().filter(|c| c.kind() == Spacer::KIND).count();
    if count == 0 {
        return size;
    }

    match direction {
        Direction::Horizontal => constraints.max_width /= count,
        Direction::Vertical => constraints.max_height /= count,
    };
    constraints.min_width = constraints.max_width;
    constraints.min_height = constraints.max_height;

    for spacer in spacers {
        // Ignore all widgets that aren't spacers
        if spacer.kind() != Spacer::KIND {
            continue;
        }

        let s = spacer.layout(constraints, force_layout);

        match direction {
            Direction::Horizontal => {
                size.width += s.width;
                size.height = size.height.max(s.height);
            }
            Direction::Vertical => {
                size.height += s.height;
                size.width = size.width.max(s.width);
            }
        }
    }

    size
}
