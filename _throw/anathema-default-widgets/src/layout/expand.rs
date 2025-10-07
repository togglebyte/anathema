use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::LayoutForEach;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx};

use super::Axis;

const DEFAULT_FACTOR: u16 = 1;

/// Distributes the total size over a list of weights
///
/// It uses the [Huntington-Hill method](https://en.wikipedia.org/wiki/Huntington%E2%80%93Hill_method)
///
/// Panics when called with more weights than the total number of available size.
/// Allocates a minimum of one to each weight.
fn distribute_size(weights: &[u16], mut total: u16) -> Vec<u16> {
    let mut indexed = weights
        .iter()
        .copied()
        .enumerate()
        .map(|(i, w)| (i, w, 0u16))
        .collect::<Vec<_>>();

    fn pop(n: &mut u16) -> bool {
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
    nodes: &mut LayoutForEach<'_, 'bp>,
    constraints: Constraints,
    axis: Axis,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Result<Size> {
    let mut factors = vec![];

    _ = nodes.each(ctx, |ctx, node, _children| {
        if node.ident == "expand" {
            let attributes = ctx.attribute_storage.get(node.id());
            let factor = attributes.get_as::<u16>("factor").unwrap_or(DEFAULT_FACTOR);
            factors.push(factor);
        }

        Ok(ControlFlow::Continue(()))
    })?;

    let mut size = Size::ZERO;

    if factors.is_empty() {
        return Ok(size);
    }

    // Distribute the available space
    let sizes = match axis {
        Axis::Horizontal => distribute_size(&factors, constraints.max_width()),
        Axis::Vertical => distribute_size(&factors, constraints.max_height()),
    };

    let mut index = 0;
    _ = nodes.each(ctx, |ctx, node, children| {
        if node.ident != "expand" {
            return Ok(ControlFlow::Continue(()));
        }

        let sub_size = sizes[index];
        index += 1;

        let constraints = match axis {
            Axis::Horizontal => {
                let mut constraints = Constraints::new(sub_size, constraints.max_height());

                // Ensure that the rounding doesn't push the constraint outside of the max width
                constraints.min_width = constraints.min_width.min(constraints.max_width());
                constraints
            }
            Axis::Vertical => {
                let mut constraints = Constraints::new(constraints.max_width(), sub_size);

                // Ensure that the rounding doesn't push the constraint outside of the max height
                constraints.min_height = constraints.min_height.min(constraints.max_height());
                constraints
            }
        };

        let widget_size = Size::from(node.layout(children, constraints, ctx)?);

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

        Ok(ControlFlow::Continue(()))
    })?;

    Ok(size)
}
