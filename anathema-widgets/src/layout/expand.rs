use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Constraints};
use anathema_widget_core::{WidgetContainer, Nodes};

use crate::Expand;

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
        indexed.sort_by_cached_key(|&(_, w, r)| {
            (((w as f64) / ((r * (r + 1)) as f64).sqrt()) * -10000.) as isize
        });
        indexed[0].2 += 1;
    }

    indexed.sort_by_key(|&(i, ..)| i);
    indexed.into_iter().map(|(_, _, r)| r).collect()
}

pub fn layout(
    ctx: &mut LayoutCtx,
    children: &mut Nodes,
    axis: Axis,
    data: Context<'_, '_>,
) -> Result<Size> {
    let expansions = children
        .iter_mut()
        .filter(|(c, _)| c.kind() == Expand::KIND)
        .collect::<Vec<_>>();

    let factors = expansions
        .iter()
        .map(|(w, _)| w.to_ref::<Expand>().factor)
        .collect::<Vec<_>>();

    let mut size = Size::ZERO;

    if factors.is_empty() {
        return Ok(size);
    }

    // Distribute the available space
    let sizes = match axis {
        Axis::Horizontal => distribute_size(&factors, ctx.constraints.max_width),
        Axis::Vertical => distribute_size(&factors, ctx.constraints.max_height),
    };

    for (sub_size, (expanded_widget, nodes)) in std::iter::zip(sizes, expansions) {
        let constraints = match axis {
            Axis::Horizontal => {
                let mut constraints = Constraints::new(sub_size, ctx.constraints.max_height);

                // Ensure that the rounding doesn't push the constraint outside of the max width
                constraints.min_width = constraints.max_width;
                constraints
            }
            Axis::Vertical => {
                let mut constraints = Constraints::new(ctx.constraints.max_width, sub_size);

                // Ensure that the rounding doesn't push the constraint outside of the max height
                constraints.min_height = constraints.max_height;
                constraints
            }
        };

        let context = Context::new(data.state, data.scope);
        let widget_size = expanded_widget.layout(nodes, constraints, &context)?;

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
    }

    Ok(size)
}
