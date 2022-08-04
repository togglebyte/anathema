use super::Constraints;
use crate::widgets::{Spacer, Axis, WidgetContainer};
use crate::display::Size;

pub fn layout(
    spacers: &mut [WidgetContainer],
    mut constraints: Constraints,
    force_layout: bool,
    direction: Axis,
) -> Size {
    let mut size = Size::ZERO;
    let count = spacers.iter_mut().filter(|c| c.kind() == Spacer::KIND).count();
    if count == 0 {
        return size;
    }

    match direction {
        Axis::Horizontal => constraints.max_width /= count,
        Axis::Vertical => constraints.max_height /= count,
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
            Axis::Horizontal => {
                size.width += s.width;
                size.height = size.height.max(s.height);
            }
            Axis::Vertical => {
                size.height += s.height;
                size.width = size.width.max(s.width);
            }
        }
    }

    size
}
