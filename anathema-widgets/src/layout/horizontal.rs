use anathema_render::Size;

use super::{expand, spacers, Constraints, Layout, Padding};
use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::{Error, Result};
use crate::gen::generator::Generator;
use crate::{Axis, Expand, Spacer, WidgetContainer};

pub struct Horizontal;

impl Layout for Horizontal {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        let mut used_width = 0;
        let mut height = 0;

        let constraints = ctx.padded_constraints();
        let max_width = constraints.max_width;

        let mut values = ctx.values.next();
        let mut gen = Generator::new(ctx.templates, ctx.lookup, &mut values);

        while let Some(mut widget) = gen.next(&mut values).transpose()? {
            let index = ctx.children.len();
            ctx.children.push(widget);
            let widget = &mut ctx.children[index];

            // Ignore spacers
            if widget.kind() == Spacer::KIND {
                continue;
            }

            // Ignore expanded widgets
            if widget.kind() == Expand::KIND {
                continue;
            }

            let constraints = Constraints::new(max_width - used_width, constraints.max_height);

            let size = match widget.layout(constraints, &values, ctx.lookup) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => break,
                err @ Err(_) => err?,
            };

            height = height.max(size.height);
            used_width = (used_width + size.width).min(max_width);

            if used_width >= max_width {
                break;
            }
        }

        if !ctx.constraints.is_width_unbounded() {
            ctx.constraints.max_width -= used_width;

            let expanded_size = expand::layout(ctx, Axis::Horizontal)?;
            height = height.max(expanded_size.height);
            used_width += expanded_size.width;

            ctx.constraints.max_height = height;
            let spacer_size = spacers::layout(ctx, Axis::Horizontal)?;
            used_width += spacer_size.width;
        }

        height = height.max(ctx.constraints.min_height) + ctx.padding.top + ctx.padding.bottom;
        height = height.min(ctx.constraints.max_height);
        size.height = size.height.max(height);

        used_width = used_width.max(ctx.constraints.min_width);

        size.width = size.width.max(used_width);

        Ok(())
    }
}

// // TODO: ignore_spacers_and_expansions is super gross, don't do this
// pub fn layout(
//     mut ctx: LayoutCtx,
//     ignore_spacers_and_expansions: bool,
//     children: &mut WidgetContainer<'_>,
// ) -> Size {
//     // let mut used_width = 0;
//     // let mut height = 0;

//     // let constraints = ctx.padded_constraints();
//     // let max_width = constraints.max_width;

//     // while let Some(mut widget) = children.next(&mut ctx.gen) {
//     //     // Ignore spacers
//     //     if widget.kind() == Spacer::KIND {
//     //         continue;
//     //     }

//     //     // Ignore expanded widgets
//     //     if widget.kind() == Expand::KIND {
//     //         continue;
//     //     }

//     //     let constraints = Constraints::new(max_width - used_width, constraints.max_height);
//     //     let size = widget.layout(constraints, ctx.ctx, ctx.lookup);

//     //     height = height.max(size.height);
//     //     used_width = (used_width + size.width).min(max_width);

//     //     if used_width >= max_width {
//     //         break
//     //     }
//     // }

//     // let expanded_size = match ignore_spacers_and_expansions {
//     //     false => expand::layout(
//     //         &mut children.widgets,
//     //         Constraints::new(max_width - used_width, ctx.constraints.max_height),
//     //         ctx.ctx,
//     //         ctx.lookup,
//     //         Direction::Horizontal,
//     //     ),
//     //     true => Size::ZERO,
//     // };

//     // height = height.max(expanded_size.height) + ctx.padding.top + ctx.padding.bottom;
//     // height = height.min(ctx.constraints.max_height);

//     // // let spacers_size = spacers::layout(
//     // //     widgets,
//     // //     Constraints::new(max_width - used_width, height),
//     // //     Direction::Horizontal,
//     // //     sub,
//     // // );
//     // let spacers_size = Size::ZERO;

//     // let mut width = used_width + expanded_size.width + spacers_size.width;
//     // width = width.max(ctx.constraints.min_width).min(ctx.constraints.max_width);

//     // Size::new(width, height)
// }

pub fn position(ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
    let mut pos = ctx.pos;
    for widget in children {
        widget.position(pos);
        pos.x += widget.outer_size().width as i32;
    }
}
