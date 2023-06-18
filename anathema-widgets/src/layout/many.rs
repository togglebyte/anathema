use anathema_render::Size;

use super::Layout;
use crate::contexts::LayoutCtx;
use crate::error::{Error, Result};
use crate::gen::generator::Generator;
use crate::{Axis, Constraints, Direction, Expand, Spacer};

pub struct Many {
    pub offset: isize,
    pub direction: Direction,
    pub axis: Axis,
    offsetting: bool,
}

impl Many {
    pub fn new(direction: Direction, axis: Axis, offset: isize) -> Self {
        Self {
            direction,
            axis,
            offset,
            offsetting: true,
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
        let max_constraints = ctx.padded_constraints();
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

            let size = match widget.layout(max_constraints, &values, ctx.lookup) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => break,
                err @ Err(_) => err?,
            };

            match self.axis {
                Axis::Vertical => {
                    if self.offsetting {
                        if self.offset >= size.height as isize {
                            self.offset -= size.height as isize;
                            ctx.children.remove(index);
                            continue
                        } else {
                            self.offsetting = false;
                            used_size.width = used_size.width.max(size.width);
                            used_size.height = (used_size.height + size.height - self.offset as usize).min(max_constraints.max_height);
                        }
                    } else {
                        used_size.width = used_size.width.max(size.width);
                        used_size.height = (used_size.height + size.height).min(max_constraints.max_height);
                    }

                    if used_size.height >= max_constraints.max_height {
                        break;
                    }
                }
                Axis::Horizontal => {
                    if self.offsetting {
                        if self.offset >= size.width as isize {
                            self.offset -= size.width as isize;
                            ctx.children.remove(index);
                            continue
                        } else {
                            self.offsetting = false;
                            used_size.width = (used_size.width + size.width - self.offset as usize).min(max_constraints.max_width);
                            used_size.height = used_size.height.max(size.height);
                        }
                    } else {
                        used_size.width = (used_size.width + size.width).min(max_constraints.max_width);
                        used_size.height = used_size.height.max(size.height);
                    }

                    if used_size.width >= max_constraints.max_width {
                        break;
                    }
                }
            }
        }

        match self.axis {
            Axis::Vertical => {
                size.width += ctx.padding.left + ctx.padding.right;
                size.width = size.width.max(used_size.width);
                size.height = size
                    .height
                    .max(used_size.height)
                    .max(max_constraints.min_height);
            }
            Axis::Horizontal => {
                size.height += ctx.padding.left + ctx.padding.right;
                size.height = size.height.max(used_size.height);
                size.width = size.width.max(used_size.width).max(max_constraints.min_width);
            }
        }

        Ok(())
    }
}
