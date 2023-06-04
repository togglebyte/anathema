// use anathema_render::Size;

// use super::{Constraints, Padding};
// use crate::ctx::{LayoutCtx, PositionCtx};
// use crate::error::Result;
// use crate::values::Layout;
// use crate::{Axis, Values, WidgetContainer, WidgetLookup, WidgetTemplate};

// // TODO: ignore_spacers_and_expansions is super gross, don't do this
// pub fn layout<'tpl>(
//     templates: &'tpl [WidgetTemplate],
//     values: &Values<'_, Layout<'_>>,
//     layout_ctx: LayoutCtx,
//     lookup: &'tpl WidgetLookup<'tpl>,
//     children: &mut Vec<WidgetContainer<'tpl>>,
//     ignore_spacers_and_expansions: bool,
// ) -> Result<Size> {
//     let mut used_height = 0;
//     let mut width = 0;

//     let constraints = layout_ctx.padded_constraints();
//     let max_height = constraints.max_height;

//     let mut values = values.scoped();
//     let mut widgets = Widgets::new(templates, lookup);

//     while let Some(mut widget) = widgets.next(&mut values).transpose()? {
//         // Ignore spacers
//         // if widget.kind() == Spacer::KIND {
//         //     continue;
//         // }

//         // // Ignore expanded
//         // if widget.kind() == Expand::KIND {
//         //     continue;
//         // }

//         let constraints = Constraints::new(constraints.max_width, max_height - used_height);
//         let values = values.layout();

//         let size = widget.layout(constraints, &values, lookup)?;
//         children.push(widget);

//         width = width.max(size.width);
//         used_height = (used_height + size.height).min(max_height);

//         if used_height >= max_height {
//             break;
//         }
//     }

//     // TODO: add this back in
//     // let expanded_size = match ignore_spacers_and_expansions {
//     //     false => expand::layout(
//     //         &mut children.widgets,
//     //         Constraints::new(ctx.constraints.max_width, max_height - used_height),
//     //         ctx.ctx,
//     //         ctx.lookup,
//     //         Direction::Vertical,
//     //     ),
//     //     true => Size::ZERO,
//     // };

//     let expanded_size = Size::ZERO;

//     width = width
//         .max(expanded_size.width)
//         .max(layout_ctx.constraints.min_width)
//         + layout_ctx.padding.left
//         + layout_ctx.padding.right;
//     width = width.min(layout_ctx.constraints.max_width);

//     // let spacers_size = spacers::layout(
//     //     widgets,
//     //     Constraints::new(width, max_height - used_height),
//     //     Direction::Vertical,
//     //     sub,
//     // );
//     let spacers_size = Size::ZERO;

//     let mut height = used_height + expanded_size.height + spacers_size.height;
//     height = height
//         .max(layout_ctx.constraints.min_height)
//         .min(layout_ctx.constraints.max_height);

//     Ok(Size::new(width, height))
// }

// pub fn position(ctx: PositionCtx, children: &mut [WidgetContainer<'_>]) {
//     let mut pos = ctx.padded_position();
//     for widget in children {
//         widget.position(pos);
//         pos.y += widget.size.height as i32;
//     }
// }
