use std::borrow::Cow;

use anathema_render::Size;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::gen::store::Store;
use crate::gen::ValueRef;
use crate::values::{Layout, Scoped};
use crate::{Constraints, Direction, Offset, Value, WidgetContainer};

#[derive(Debug, Copy, Clone)]
enum Fix {
    Pre,
    Post,
}

pub(super) struct ViewportLayout<'widget, 'tpl, 'parent> {
    offset: Offset,
    layout: LayoutCtx<'widget, 'tpl, 'parent>,
    constraints: Constraints,
    binding: &'widget str,
    max_height: usize,
    size: Size,
    non_repeating_widget_count: usize,
    widget_count: usize,
}

impl<'widget, 'tpl, 'parent> ViewportLayout<'widget, 'tpl, 'parent> {
    pub fn new(
        offset: Offset,
        mut layout: LayoutCtx<'widget, 'tpl, 'parent>,
        binding: &'widget str,
    ) -> Self {
        let mut constraints = layout.padded_constraints();
        let max_height = constraints.max_height;
        constraints.unbound_height();

        Self {
            layout,
            offset,
            binding,
            constraints,
            max_height,
            size: Size::ZERO,
            non_repeating_widget_count: 0,
            widget_count: 0,
        }
    }

    pub(super) fn layout(&mut self, data: &'parent [Value], direction: Direction) -> Result<Size> {
        let mut values = self.layout.values.next();
        let mut gen = Generator::new(self.layout.templates, self.layout.lookup, &mut values);
        // gen.skip(self.offset.element, &mut values);

        while let Some(widget) = gen.next(&mut values).transpose()? {
            self.layout_and_add_widget(widget, data, &values, direction, Fix::Post)?;
        }

        // gen.reverse();
        // if let Some(widget) = gen.next(&mut values, self.layout.lookup).transpose()? {
        //     self.layout_and_add_widget(widget, data, &values, direction, Fix::Post)?;
        // }
        // if let Some(widget) = gen.next(&mut values, self.layout.lookup).transpose()? {
        //     self.layout_and_add_widget(widget, data, &values, direction, Fix::Post)?;
        // }


        // if self.offset.cell < 0 {
        //     gen.reverse();
        //     while self.offset.cell < 0 {
        //         if let Some(widget) = gen.next(&mut values, self.layout.lookup).transpose()? {
        //             self.layout_and_add_widget(widget, data, &values, direction, Fix::Post)?;

        //             if self.size.height >= self.max_height {
        //                 break;
        //             }
        //         }
        //     }
        // }

        // * Step skip until offset.element count == 0
        // * Reverse if the offset.cell count < 0
        //     * Count generated widgets
        //     * Reverse
        //     * Skip generated_widget_count
        // * Finally: get on with it

        // while let Some(widget) = gen.next(&mut values, self.layout.lookup).transpose()? {
        //     self.layout_and_add_widget(widget, data, &values, direction, Fix::Post)?;
        //     if self.size.height >= self.max_height {
        //         break;
        //     }
        // }

        Ok(self.size)
    }

    fn layout_and_add_widget(
        &mut self,
        widget: WidgetContainer<'tpl>,
        data: &'parent [Value],
        values: &Store<'_>,
        direction: Direction,
        fix: Fix,
    ) -> Result<()> {
        self.layout_widget(widget, values.next(), fix)?;
        Ok(())
    }

    fn layout_widget(
        &mut self,
        mut widget: WidgetContainer<'tpl>,
        values: Store<'_>,
        fix: Fix,
    ) -> Result<()> {
        // Ignore spacers
        // if widget.kind() == Spacer::KIND {
        //     return Ok(());
        // }

        // // Ignore expanded
        // if widget.kind() == Expand::KIND {
        //     return Ok(());
        // }

        let size = widget.layout(self.constraints, &values, self.layout.lookup)?;
        self.apply_size(size);

        match fix {
            Fix::Pre => self.layout.children.insert(0, widget),
            Fix::Post => self.layout.children.push(widget),
        }

        Ok(())
    }

    fn apply_size(&mut self, size: Size) {
        self.size.width = self.size.width.max(size.width);
        self.size.height += size.height;
        if self.size.height > self.max_height {
            self.size.height = self.max_height;
        }
    }
}
