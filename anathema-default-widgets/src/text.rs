use std::ops::ControlFlow;

use anathema_geometry::{LocalPos, Size};
use anathema_widgets::layout::{Constraints, IterEntry, LayoutCtx, PositionCtx, TextBuffer, TextIndex};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId};

use crate::layout::text::ProcessResult::{Continue, Done};
use crate::layout::text::{Lines, TextAlignment, TextLayout};

/// Text widget
/// ```ignore
/// Attributes:
/// * background
/// * foreground
/// * text-align
/// * wrap
/// ```
///
/// Note: Spans, unlike other widgets, does not require a widget id
///
/// A `Text` widget will be as wide as its text.
#[derive(Debug, Default)]
pub struct Text {
    text_key: TextIndex,
}

impl Widget for Text {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let attributes = ctx.attribs.get(id);
        let wrap = attributes.get_c("wrap").unwrap_or_default();
        let mut session = ctx.text.new_session();
        self.text_key = session.new_key();

        let mut layout = TextLayout::new(constraints.max_size(), wrap, session, self.text_key);
        layout.set_style(id);

        // Layout text
        attributes.value().map(|value| {
            value.str_iter(|s| match layout.process(s) {
                Done => ControlFlow::Break(()),
                Continue => ControlFlow::Continue(()),
            })
        });

        // Layout text of all the sub-nodes
        children.for_each(|child, _| {
            let Some((_span, widget_id)) = child.to_inner_mut::<Span>() else {
                return ControlFlow::Continue(());
            };
            layout.set_style(widget_id);

            let attributes = ctx.attribs.get(widget_id);
            if let Some(text) = attributes.value() {
                text.str_iter(|s| match layout.process(s) {
                    Done => ControlFlow::Break(()),
                    Continue => ControlFlow::Continue(()),
                })?;

                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        });

        layout.finish();
        layout.size()
    }

    fn paint<'bp>(
        &mut self,
        _: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
        text_buffer: &mut TextBuffer,
    ) {
        let session = text_buffer.new_session();
        let lines = Lines::new(self.text_key, session);
        let alignment: TextAlignment = attribute_storage.get(id).get_c("text-align").unwrap_or_default();

        let mut pos = LocalPos::ZERO;
        let mut style = attribute_storage.get(id);

        for line in lines.iter() {
            let width = line.width as u16;
            let x = match alignment {
                TextAlignment::Left => 0,
                TextAlignment::Centre => ctx.local_size.width as u16 / 2 - width / 2,
                TextAlignment::Right => ctx.local_size.width as u16 - width,
            };

            pos.x = x;

            for entry in line.iter {
                match entry {
                    IterEntry::Str(s) => {
                        if let Some(new_pos) = ctx.place_glyphs(s, style, pos) {
                            pos = new_pos;
                        }
                    }
                    IterEntry::Style(attribute_id) => style = attribute_storage.get(attribute_id),
                }
            }
            pos.y += 1;
            pos.x = 0;
        }
    }

    fn position<'bp>(
        &mut self,
        _children: PositionChildren<'_, '_, 'bp>,
        _attributes: WidgetId,
        _attribute_storage: &AttributeStorage<'bp>,
        _ctx: PositionCtx,
    ) {
        // NOTE: there is no need to position text as the text
        // is printed from the context position
    }
}

#[derive(Default, Copy, Clone)]
pub struct Span;

impl Widget for Span {
    fn layout<'bp>(
        &mut self,
        _: LayoutChildren<'_, '_, 'bp>,
        _: Constraints,
        _: WidgetId,
        _: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        // Everything is handled by the parent text
        panic!("this should never be called");
    }

    fn position<'bp>(
        &mut self,
        _: PositionChildren<'_, '_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PositionCtx,
    ) {
        // Everything is handled by the parent text
        panic!("this should never be called");
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn word_wrap_excessive_space() {
        let src = "text 'hello      how are     you'";
        let expected = "
           ╔════════════════╗
           ║hello      how  ║
           ║are     you     ║
           ║                ║
           ║                ║
           ║                ║
           ║                ║
           ╚════════════════╝";

        TestRunner::new(src, (16, 6)).instance().render_assert(expected);
    }

    #[test]
    fn word_wrap() {
        let src = "text 'hello how are you'";
        let expected = r#"
           ╔════════════════╗
           ║hello how are   ║
           ║you             ║
           ║                ║
           ╚════════════════╝
           "#;

        TestRunner::new(src, (16, 3)).instance().render_assert(expected);
    }

    #[test]
    fn no_word_wrap() {
        let src = "text [wrap: 'overflow'] 'hello how are you'";
        let expected = r#"
           ╔════════════════╗
           ║hello how are yo║
           ║                ║
           ║                ║
           ╚════════════════╝
           "#;

        TestRunner::new(src, (16, 3)).instance().render_assert(expected);
    }

    #[test]
    fn break_word_wrap() {
        let src = "text [wrap: 'break'] 'hellohowareyoudoing'";
        let expected = r#"
           ╔════════════════╗
           ║hellohowareyoudo║
           ║ing             ║
           ║                ║
           ╚════════════════╝
           "#;

        TestRunner::new(src, (16, 3)).instance().render_assert(expected);
    }

    #[test]
    fn char_wrap_layout_multiple_spans() {
        let src = r#"
            text 'one'
                span 'two'
                span ' averylongword'
                span ' bunny'
        "#;

        let expected = r#"
           ╔═══════════════════╗
           ║onetwo             ║
           ║averylongword bunny║
           ║                   ║
           ╚═══════════════════╝
       "#;

        TestRunner::new(src, (19, 3)).instance().render_assert(expected);
    }

    #[test]
    fn multi_line_with_span() {
        let src = r#"
            border [width: 5 + 2]
                text 'one'
                    span 'two'
        "#;

        let expected = r#"
            ╔═════════╗
            ║┌─────┐  ║
            ║│onetw│  ║
            ║│o    │  ║
            ║└─────┘  ║
            ╚═════════╝
       "#;

        TestRunner::new(src, (9, 4)).instance().render_assert(expected);
    }

    #[test]
    fn right_alignment() {
        let src = "text [text-align: 'right'] 'a one xxxxxxxxxxxxxxxxxx'";
        let expected = r#"
               ╔══════════════════╗
               ║            a one ║
               ║xxxxxxxxxxxxxxxxxx║
               ║                  ║
               ╚══════════════════╝
           "#;

        TestRunner::new(src, (18, 3)).instance().render_assert(expected);
    }

    #[test]
    fn centre_alignment() {
        let src = "text [text-align: 'centre'] 'a one xxxxxxxxxxxxxxxxxx'";
        let expected = r#"
               ╔══════════════════╗
               ║      a one       ║
               ║xxxxxxxxxxxxxxxxxx║
               ║                  ║
               ╚══════════════════╝
           "#;

        TestRunner::new(src, (18, 3)).instance().render_assert(expected);
    }
}
