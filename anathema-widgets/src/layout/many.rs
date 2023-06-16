use anathema_render::Size;

use super::Layout;
use crate::contexts::LayoutCtx;
use crate::error::{Error, Result};
use crate::gen::generator::Generator;
use crate::{Axis, Constraints, Direction, Expand, Spacer};

pub struct Many {
    offset: usize,
    direction: Direction,
    axis: Axis,
}

impl Many {
    pub fn new(direction: Direction, axis: Axis, offset: usize) -> Self {
        Self {
            direction,
            axis,
            offset
        }
    }
}

impl Layout for Many {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        let mut values = ctx.values.next();
        let mut gen = Generator::new(ctx.templates, ctx.lookup, &mut values);
        let constraints = ctx.padded_constraints();
        let mut used_size = Size::ZERO;

        if let Direction::Backward = self.direction {
            gen.flip();
        }

        while let Some(mut widget) = gen.next(&mut values).transpose()? {
            let index = ctx.children.len();
            ctx.children.push(widget);
            let widget = &mut ctx.children[index];

            // Ignore spacers
            if widget.kind() == Spacer::KIND {
                continue;
            }

            // Ignore expanded
            if widget.kind() == Expand::KIND {
                continue;
            }

            let constraints = match self.axis {
                Axis::Vertical => Constraints::new(
                    constraints.max_width,
                    constraints.max_height - used_size.height,
                ),
                Axis::Horizontal => Constraints::new(
                    constraints.max_width - used_size.width,
                    constraints.max_height,
                ),
            };

            let size = match widget.layout(constraints, &values, ctx.lookup) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => break,
                err @ Err(_) => err?,
            };

            match self.axis {
                Axis::Vertical => {
                    used_size.width = used_size.width.max(size.width);
                    used_size.height = (used_size.height + size.height).min(constraints.max_height);

                    self.offset = self.offset.saturating_sub(size.height);
                    if self.offset > 0 {
                        ctx.children.remove(index);
                    }

                    if used_size.width >= constraints.max_width {
                        break;
                    }
                }
                Axis::Horizontal => {
                    used_size.width = (used_size.width + size.width).min(constraints.max_width);
                    used_size.height = used_size.height.max(size.height);

                    self.offset = self.offset.saturating_sub(size.width);
                    if self.offset > 0 {
                        ctx.children.remove(index);
                    }

                    if used_size.height >= constraints.max_height {
                        break;
                    }
                }
            }
        }

        match self.axis {
            Axis::Vertical => {
                size.width += ctx.padding.left + ctx.padding.right;
                size.width = size.width.max(used_size.width);
                size.height = size.height.max(used_size.height).max(constraints.min_height);
            }
            Axis::Horizontal => {
                size.height += ctx.padding.left + ctx.padding.right;
                size.height = size.height.max(used_size.height);
                size.width = size.width.max(used_size.width).max(constraints.min_width);
            }
        }

        Ok(())
    }
}
