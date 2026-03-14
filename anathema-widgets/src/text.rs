use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, LayoutSize, Layouts, Widget};
use anathema_runtime::{Constraints, WidgetAttributes};
use unicode_width::UnicodeWidthStr;

use crate::textlayout::{Instruction, PerformLayout, TextLayout};

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

impl Widget for Text {
    fn layout<'bp>(
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

    fn position<'bp>(&mut self, _: Children<'_, 'bp>, _: WidgetAttributes<'_, 'bp>, _: &mut Layouts, _: Pos) {}

    fn paint<'bp>(
        &mut self,
        region: Region,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        let mut text_offset = region.from;

        for (attribs, seg) in std::iter::once(attributes)
            .chain(children.map(|c| c.attributes))
            .zip(self.layout.segments.iter())
        {
            let Some(text) = attribs.value_as::<&str>() else { continue };

            for instruction in seg.instructions() {
                match instruction {
                    Instruction::Newline => {
                        text_offset.x = region.from.x;
                        text_offset.y += 1;
                    }
                    Instruction::Print(range) => {
                        let text = &text[range];

                        let width = text.width() as i32;
                        let region = Region::new(text_offset, text_offset + Pos::new(width, 1));
                        frontend.apply_brush_to_region(&attribs, region);

                        frontend.set_text(text, text_offset);
                    }
                }
            }
        }
    }

    fn describe(&self) -> &str {
        "text"
    }
}

pub struct Span {}

impl Span {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Span {
    fn layout<'bp>(
        &mut self,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        constraints: Constraints,
    ) -> LayoutSize {
        todo!()
    }

    fn position<'bp>(
        &mut self,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        pos: Pos,
    ) {
        todo!()
    }

    fn paint<'bp>(
        &mut self,
        region: Region,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        todo!()
    }

    fn describe(&self) -> &'static str {
        "span"
    }
}
