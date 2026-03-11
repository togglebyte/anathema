use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, LayoutSize, Layouts, Widget};
use anathema_runtime::{Constraints, WidgetAttributes};

use crate::textlayout::{PerformLayout, TextLayout};

pub struct Text {
    layout: TextLayout,
}

impl Text {
    pub fn new() -> Self {
        Self {
            layout: TextLayout::new(),
        }
    }
}

impl<'bp> Widget<'bp> for Text {
    fn layout(
        &mut self,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        mut constraints: Constraints,
    ) -> LayoutSize {
        let mut size = Size::ZERO;
        let Some(text) = attributes.value_as::<&str>() else { return LayoutSize::ZERO };
        let mut layout = PerformLayout::new(constraints.max);
        layout.feed(text);

        for mut child in children {
            let Some(text) = child.attributes.value_as::<&str>() else { continue };
            let Some(child) = child.to::<Span>() else { continue };
            layout.feed(text);
        }

        self.layout = layout.finish();

        LayoutSize::same(size)
    }

    fn position(&mut self, _: Children<'_, 'bp>, _: WidgetAttributes<'_, 'bp>, _: &mut Layouts, _: Pos) {}

    fn paint(
        &mut self,
        region: Region,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        // TODO: this has to be added as an offset for every position in the layou
        let text_offset = region.from;

        let Some(text) = attributes.value_as::<&str>() else { return };
        // frontend.apply_brush_to_region(&attributes, region);

        // self.text_layout
        // frontend.set_text(layout.pos, text);

        // for (layout, child) in children.zip(self.layout.children()) {
        //     let Some(text) = child.attributes.value_as::<&str>() else { continue };
        //     frontend.apply_brush_to_region(&child.attributes, layout.region);
        //     frontend.set_text(layout.pos, text);
        // }
    }

    fn describe(&self) -> &str {
        "text"
    }
}


pub struct Span {
    layout: TextLayout,
}

impl Span {
    pub fn new() -> Self {
        Self {
            layout: TextLayout::new(),
        }
    }
}

impl<'bp> Widget<'bp> for Span {
    fn layout(
        &mut self,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        constraints: Constraints,
    ) -> LayoutSize {
        todo!()
    }

    fn position(
        &mut self,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        pos: Pos,
    ) {
        todo!()
    }

    fn paint(
        &mut self,
        region: Region,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        todo!()
    }
}
