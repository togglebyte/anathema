use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx};
use anathema_widgets::LayoutChildren;

use super::Axis;

const DEFAULT_FACTOR: usize = 1;

/// Distributes the total size over a list of weights
///
/// It uses the [Huntington-Hill method](https://en.wikipedia.org/wiki/Huntington%E2%80%93Hill_method)
///
/// Panics when called with more weights than the total number of available size.
/// Allocates a minimum of one to each weight.
fn distribute_size(weights: &[usize], mut total: usize) -> Vec<usize> {
    assert!(total > weights.len());

    let mut indexed = weights
        .iter()
        .copied()
        .enumerate()
        .map(|(i, w)| (i, w, 1usize))
        .collect::<Vec<_>>();

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

pub fn layout_all_expansions<'bp>(
    nodes: &mut LayoutChildren<'_, '_, 'bp>,
    constraints: Constraints,
    axis: Axis,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Size {
    let mut factors = vec![];

    nodes.for_each(|node, _children| {
        if node.ident == "expand" {
            let attributes = ctx.attribs.get(node.id());
            let factor = attributes.get("factor").unwrap_or(DEFAULT_FACTOR);
            factors.push(factor);
        }

        ControlFlow::Continue(())
    });

    let mut size = Size::ZERO;

    if factors.is_empty() {
        return size;
    }

    // Distribute the available space
    let sizes = match axis {
        Axis::Horizontal => distribute_size(&factors, constraints.max_width()),
        Axis::Vertical => distribute_size(&factors, constraints.max_height()),
    };

    let mut index = 0;
    nodes.for_each(|node, children| {
        if node.ident != "expand" {
            return ControlFlow::Continue(());
        }

        let sub_size = sizes[index];
        index += 1;

        let constraints = match axis {
            Axis::Horizontal => {
                let mut constraints = Constraints::new(sub_size, constraints.max_height());

                // Ensure that the rounding doesn't push the constraint outside of the max width
                constraints.min_width = constraints.max_width();
                constraints
            }
            Axis::Vertical => {
                let mut constraints = Constraints::new(constraints.max_width(), sub_size);

                // Ensure that the rounding doesn't push the constraint outside of the max height
                constraints.min_height = constraints.max_height();
                constraints
            }
        };

        let widget_size = node.layout(children, constraints, ctx);

        match axis {
            Axis::Horizontal => {
                size.width += widget_size.width;
                size.height = size.height.max(widget_size.height);
            }
            Axis::Vertical => {
                size.width = size.width.max(widget_size.width);
                size.height += widget_size.height;
            }
        }

        ControlFlow::Continue(())
    });

    size
}
