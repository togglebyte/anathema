use crate::display::Size;

use super::Constraints;
use crate::widgets::ctx::LayoutCtx;
use crate::widgets::{Direction, Expand, WidgetContainer};

fn distribute_size(weights: &[usize], mut total: usize) -> Vec<usize> {
    assert!(total > weights.len());
    let mut indexed = weights.iter().copied().enumerate().map(|(i, w)| (i, w, 1usize)).collect::<Vec<_>>();
    total -= weights.len();

    fn pop(n: &mut usize) -> bool {
        if let Some(nn) = n.checked_sub(1) {
            *n = nn;
            true
        } else {
            false
        }
    }

    while pop(&mut total) {
        indexed.sort_by_cached_key(|&(_, w, r)| (((w as f64) / ((r * (r + 1)) as f64).sqrt()) * -10000.) as isize);
        indexed[0].2 += 1;
    }

    indexed.sort_by_key(|&(i, ..)| i);
    indexed.into_iter().map(|(_, _, r)| r).collect()
}

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx, direction: Direction) -> Size {
    let mut expansions = widgets.iter_mut().filter(|c| c.kind() == Expand::KIND).collect::<Vec<_>>();
    let factors = expansions.iter_mut().map(|w| w.to::<Expand>().factor).collect::<Vec<_>>();

    let mut size = Size::ZERO;

    if factors.is_empty() {
        return size;
    }

    // Distribute the available space
    let sizes = match direction {
        Direction::Horizontal => distribute_size(&factors, ctx.constraints.max_width),
        Direction::Vertical => distribute_size(&factors, ctx.constraints.max_height),
    };

    for (sub_size, expanded_widget) in std::iter::zip(sizes, expansions) {
        let constraints = match direction {
            Direction::Horizontal => {
                let mut constraints = Constraints::new(sub_size, ctx.constraints.max_height);

                // Ensure that the rounding doesn't push the constraint outside of the max width
                if constraints.max_width + size.width > ctx.constraints.max_width {
                    constraints.max_width = ctx.constraints.max_width - size.width;
                }

                constraints.min_width = constraints.max_width;
                constraints
            }
            Direction::Vertical => {
                let mut constraints = Constraints::new(ctx.constraints.max_width, sub_size);

                // Ensure that the rounding doesn't push the constraint outside of the max height
                if constraints.max_height + size.height > ctx.constraints.max_height {
                    constraints.max_height = ctx.constraints.max_height - size.height;
                }

                constraints.min_height = constraints.max_height;
                constraints
            }
        };

        let widget_size = expanded_widget.layout(constraints, ctx.force_layout);

        match direction {
            Direction::Horizontal => {
                size.width += widget_size.width;
                size.height = size.height.max(widget_size.height);
            }
            Direction::Vertical => {
                size.width = size.width.max(widget_size.width);
                size.height += widget_size.height;
            }
        }
    }

    size
}
