use display::Size;

use crate::ctx::LayoutCtx;
use crate::layout::Constraints;
use crate::widgets::expanded::Expand;
use crate::widgets::{Axis, WidgetContainer};

pub fn layout(widgets: &mut [WidgetContainer], ctx: LayoutCtx, direction: Axis) -> Size {
    let mut expansions = widgets
        .iter_mut()
        .filter(|c| c.kind() == Expand::KIND)
        .collect::<Vec<_>>();
    let factors = expansions
        .iter_mut()
        .map(|w| w.to::<Expand>().factor)
        .sum::<usize>() as f32;

    let mut size = Size::ZERO;

    if factors == 0f32 {
        return size;
    }

    for expanded_widget in expansions {
        let factor = expanded_widget.to::<Expand>().factor as f32;
        let constraints = match direction {
            Axis::Horizontal => {
                let width_per_factor = ctx.constraints.max_width as f32 / factors;
                let mut constraints = Constraints::new(
                    (width_per_factor * factor).round() as usize,
                    ctx.constraints.max_height,
                );

                // Ensure that the rounding doesn't push the constraint outside of the max width
                if constraints.max_width + size.width > ctx.constraints.max_width {
                    constraints.max_width = ctx.constraints.max_width - size.width;
                }
                constraints.min_width = constraints.max_width;
                constraints
            }
            Axis::Vertical => {
                let height_per_factor = ctx.constraints.max_height as f32 / factors;
                let mut constraints = Constraints::new(
                    ctx.constraints.max_width,
                    (height_per_factor * factor).round() as usize,
                );

                // Ensure that the rounding doesn't push the constraint outside of the max height
                if constraints.max_height + size.height > ctx.constraints.max_height {
                    constraints.max_height = ctx.constraints.max_height - size.height;
                }
                constraints.min_height = constraints.max_height;
                constraints
            },
        };

        let widget_size = expanded_widget.layout(constraints, ctx.force_layout);

        match direction {
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

    size
}
