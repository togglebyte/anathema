use std::collections::HashMap;

use anathema_render::Screen;

use crate::contexts::{DataCtx, LayoutCtx};
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::gen::store::Store;
use crate::template::Template;
use crate::{Constraints, Lookup, Padding, PaintCtx, Pos, Value};

pub trait View {
    fn update(&mut self) {}

    fn templates(&self) -> &[Template];

    fn ctx(&self) -> &DataCtx;

    fn render(
        &mut self,
        lookup: &Lookup,
        constraints: Constraints,
        screen: &mut Screen,
    ) -> Result<()> {
        let mut frame = vec![];
        let mut values = Store::new(self.ctx());

        let layout_args = LayoutCtx::new(
            self.templates(),
            &values,
            constraints,
            Padding::ZERO,
            &mut frame,
            lookup,
        );

        let mut widgets = Generator::new(self.templates(), lookup, &mut values);
        while let Some(mut widget) = widgets.next(&mut values).transpose()? {
            widget.layout(constraints, &values, lookup)?;
            frame.push(widget);
        }

        for widget in &mut frame {
            widget.position(Pos::ZERO);
        }

        for mut widget in frame {
            widget.paint(PaintCtx::new(screen, None));
        }

        Ok(())
    }
}

pub struct DefaultView<F> {
    templates: Vec<Template>,
    ctx: DataCtx,
    update_fn: F,
}

impl<F> DefaultView<F>
where
    F: FnMut(&mut HashMap<String, Value>),
{
    pub fn new(templates: Vec<Template>, update_fn: F) -> Self {
        Self {
            templates,
            ctx: DataCtx::default(),
            update_fn,
        }
    }
}

impl<F> View for DefaultView<F>
where
    F: FnMut(&mut DataCtx),
{
    fn templates(&self) -> &[Template] {
        &self.templates
    }

    fn ctx(&self) -> &DataCtx {
        &self.ctx
    }

    fn update(&mut self) {
        (self.update_fn)(&mut self.ctx);
    }
}
