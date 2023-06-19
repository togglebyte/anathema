use anathema_render::Size;

use super::{Constraints, Layout, Padding};
use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::{Result, Error};
use crate::gen::generator::Generator;
use crate::{Axis, WidgetContainer};

pub struct Stacked;


//     let constraints = ctx.padded_constraints();

//     let mut height = 0;
//     let mut width = 0;

//     for widget in widgets.iter_mut() {
//         // Ignore spacers
//         if widget.kind() == Spacer::KIND {
//             continue;
//         }

//         let size = widget.layout(constraints, ctx.force_layout);

//         width = width.max(size.width);
//         height = height.max(size.height);
//     }

//     Size::new(width, height)


impl Layout for Stacked {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        let mut width = 0;
        let mut height = 0;

        let constraints = ctx.padded_constraints();
        let mut values = ctx.values.next();
        let mut gen = Generator::new(ctx.templates, ctx.lookup, &mut values);

        while let Some(mut widget) = gen.next(&mut values).transpose()? {
            let index = ctx.children.len();
            ctx.children.push(widget);
            // // Ignore spacers
            // if widget.kind() == Spacer::KIND {
            //     continue;
            // }

            // // Ignore expanded
            // if widget.kind() == Expand::KIND {
            //     continue;
            // }

            let size = match ctx.children[index].layout(constraints, &values, ctx.lookup) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => break,
                err @ Err(_) => err?,
            };

            width = width.max(size.width);
            height = height.max(size.height);
        }

        size.width = size.width.max(width);
        size.height = size.height.max(height);

        Ok(())
    }
}

// // TODO: ignore_spacers_and_expansions is super gross, don't do this
// pub fn layout<'tpl>(ctx: LayoutCtx<'_, '_, '_>) -> Result<Size> {
//     let mut used_height = 0;
//     let mut width = 0;

//     let constraints = ctx.padded_constraints();
//     let max_height = constraints.max_height;

//     let mut values = ctx.values.next();
//     let mut gen = Generator::new(ctx.templates, ctx.lookup, &mut values);

//     while let Some(mut widget) = gen.next(&mut values).transpose()? {
//         // // Ignore spacers
//         // // if widget.kind() == Spacer::KIND {
//         // //     continue;
//         // // }

//         // // // Ignore expanded
//         // // if widget.kind() == Expand::KIND {
//         // //     continue;
//         // // }

//         // let constraints = Constraints::new(constraints.max_width, max_height - used_height);
//         // let values = values.layout();

//         // // let size = widget.layout(constraints, &values, lookup)?;
//         // ctx.children.push(widget);

//         // width = width.max(size.width);
//         // used_height = (used_height + size.height).min(max_height);

//         // if used_height >= max_height {
//         //     break;
//         // }
//     }

//     // // TODO: add this back in
//     // // let expanded_size = match ignore_spacers_and_expansions {
//     // //     false => expand::layout(
//     // //         &mut children.widgets,
//     // //         Constraints::new(ctx.constraints.max_width, max_height - used_height),
//     // //         ctx.ctx,
//     // //         ctx.lookup,
//     // //         Direction::Vertical,
//     // //     ),
//     // //     true => Size::ZERO,
//     // // };

//     // let expanded_size = Size::ZERO;

//     // width = width
//     //     .max(expanded_size.width)
//     //     .max(layout_ctx.constraints.min_width)
//     //     + layout_ctx.padding.left
//     //     + layout_ctx.padding.right;
//     // width = width.min(layout_ctx.constraints.max_width);

//     // // let spacers_size = spacers::layout(
//     // //     widgets,
//     // //     Constraints::new(width, max_height - used_height),
//     // //     Direction::Vertical,
//     // //     sub,
//     // // );
//     // let spacers_size = Size::ZERO;

//     // let mut height = used_height + expanded_size.height + spacers_size.height;
//     // height = height
//     //     .max(layout_ctx.constraints.min_height)
//     //     .min(layout_ctx.constraints.max_height);

//     // Ok(Size::new(width, height))
//     panic!()
// }

pub fn position(ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
    let mut pos = ctx.pos;
    for widget in children {
        widget.position(pos);
        pos.y += widget.outer_size().height as i32;
    }
}

