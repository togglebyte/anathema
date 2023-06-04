use anathema_render::Size;

use super::{expand, spacers};
use super::{Constraints, Padding};
use crate::ctx::{LayoutCtx, PositionCtx};
use crate::{Axis, Expand, Spacer, WidgetContainer};

// TODO: ignore_spacers_and_expansions is super gross, don't do this
pub fn layout(
    mut ctx: LayoutCtx,
    ignore_spacers_and_expansions: bool,
    children: &mut WidgetContainer<'_>,
) -> Size {
    panic!()
    // let mut used_width = 0;
    // let mut height = 0;

    // let constraints = ctx.padded_constraints();
    // let max_width = constraints.max_width;

    // while let Some(mut widget) = children.next(&mut ctx.gen) {
    //     // Ignore spacers
    //     if widget.kind() == Spacer::KIND {
    //         continue;
    //     }

    //     // Ignore expanded widgets
    //     if widget.kind() == Expand::KIND {
    //         continue;
    //     }

    //     let constraints = Constraints::new(max_width - used_width, constraints.max_height);
    //     let size = widget.layout(constraints, ctx.ctx, ctx.lookup);

    //     height = height.max(size.height);
    //     used_width = (used_width + size.width).min(max_width);

    //     if used_width >= max_width {
    //         break
    //     }
    // }

    // let expanded_size = match ignore_spacers_and_expansions {
    //     false => expand::layout(
    //         &mut children.widgets,
    //         Constraints::new(max_width - used_width, ctx.constraints.max_height),
    //         ctx.ctx,
    //         ctx.lookup,
    //         Direction::Horizontal,
    //     ),
    //     true => Size::ZERO,
    // };


    // height = height.max(expanded_size.height) + ctx.padding.top + ctx.padding.bottom;
    // height = height.min(ctx.constraints.max_height);

    // // let spacers_size = spacers::layout(
    // //     widgets,
    // //     Constraints::new(max_width - used_width, height),
    // //     Direction::Horizontal,
    // //     sub,
    // // );
    // let spacers_size = Size::ZERO;

    // let mut width = used_width + expanded_size.width + spacers_size.width;
    // width = width.max(ctx.constraints.min_width).min(ctx.constraints.max_width);

    // Size::new(width, height)
}

pub fn position(ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
    let mut pos = ctx.padded_position();
    for widget in children {
        widget.position(pos);
        pos.x += widget.size.width as i32;
    }
}
