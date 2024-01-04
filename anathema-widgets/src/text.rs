use anathema_render::Size;
use anathema_values::{Context, NodeId, Value};
use anathema_widget_core::contexts::{PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, LocalPos, Nodes, Widget, WidgetFactory, WidgetStyle,
};

use crate::layout::text::{Line, ProcessOutput, TextAlignment, TextLayout, Wrap};

// -----------------------------------------------------------------------------
//     - Text -
// -----------------------------------------------------------------------------
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
#[derive(Debug)]
pub struct Text {
    /// Word wrapping
    pub word_wrap: Value<Wrap>,
    /// Text alignment. Note that text alignment only aligns the text inside the parent widget,
    /// this will not force the text to the right side of the output, for that use
    /// [`Alignment`](crate::Alignment).
    pub text_alignment: Value<TextAlignment>,
    /// Text
    pub text: Value<String>,
    /// Text style
    pub style: WidgetStyle,
    /// Squash empty lines containing a singular whitespace char
    pub squash: Value<bool>,

    layout: TextLayout,
}

impl Text {
    pub const KIND: &'static str = "Text";

    fn paint_line(
        &self,
        line: &Line,
        children: &[&TextSpan],
        y: usize,
        ctx: &mut PaintCtx<'_, WithSize>,
    ) {
        let mut pos = LocalPos::new(0, y);

        let max_width = self.layout.size().width;
        match self.text_alignment.value_or_default() {
            TextAlignment::Left => {}
            TextAlignment::Centre => pos.x = max_width / 2 - line.width / 2,
            TextAlignment::Right => pos.x = max_width - line.width,
        }

        for segment in &line.segments {
            let (text, style) = match segment.index {
                0 => (self.text.str(), self.style.style()),
                i => {
                    let child = children[i - 1];
                    let text = child.text.str();
                    let style = child.style.style();
                    (text, style)
                }
            };

            let text = segment.slice(text);
            let Some(new_pos) = ctx.print(text, style, pos) else {
                continue;
            };

            pos = new_pos;
        }
    }
}

impl Widget for Text {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.word_wrap.resolve(context, node_id);
        self.text_alignment.resolve(context, node_id);
        self.text.resolve(context, node_id);
        self.style.resolve(context, node_id);
        self.squash.resolve(context, node_id);
    }

    fn layout(&mut self, nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        let constraints = nodes.constraints;
        self.layout.reset(
            Size::new(constraints.max_width, constraints.max_height),
            self.squash.value_or(true),
        );

        self.layout.process(self.text.str());

        let _ = nodes.for_each(|mut span| {
            // Ignore any widget that isn't a span
            if span.kind() != TextSpan::KIND {
                return Ok(());
            }

            let inner_span = span.to_mut::<TextSpan>();

            match self.layout.process(inner_span.text.str()) {
                ProcessOutput::Done => Ok(()),
                ProcessOutput::InsufficientSpaceAvailble => Err(Error::InsufficientSpaceAvailble),
            }
        });

        self.layout.finish();

        let size = self.layout.size();
        Ok(size)
    }

    fn paint<'ctx>(&mut self, children: &mut Nodes<'_>, mut ctx: PaintCtx<'_, WithSize>) {
        let children = children
            .iter_mut()
            .map(|(c, _)| c.to_ref::<TextSpan>())
            .collect::<Vec<_>>();
        let lines = self.layout.lines();
        for (y, line) in lines.iter().enumerate() {
            self.paint_line(line, children.as_slice(), y, &mut ctx);
        }
    }

    fn position<'ctx>(&mut self, _: &mut Nodes<'_>, _: PositionCtx) {
        // NOTE: there is no need to position text as the text
        // is printed from the context position
    }
}

/// Represents a chunk of text with its own style
#[derive(Debug)]
pub struct TextSpan {
    /// The text
    pub text: Value<String>,
    /// Style for the text
    pub style: WidgetStyle,
}

impl TextSpan {
    const KIND: &'static str = "TextSpan";
}

impl Widget for TextSpan {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.text.resolve(context, node_id);
        self.style.resolve(context, node_id);
    }

    fn layout(&mut self, _nodes: &mut LayoutNodes<'_, '_, '_>) -> Result<Size> {
        panic!("layout should never be called directly on a span");
    }

    fn position<'ctx>(&mut self, _: &mut Nodes<'_>, _: PositionCtx) {
        // NOTE: there is no need to position text as the text is printed from the context position
        panic!("don't invoke position on the span directly.");
    }

    fn paint<'ctx>(&mut self, _: &mut Nodes<'_>, _: PaintCtx<'_, WithSize>) {
        panic!("don't invoke paint on the span directly.");
    }
}

pub(crate) struct TextFactory;

impl WidgetFactory for TextFactory {
    fn make(&self, mut ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let word_wrap = ctx.get("wrap");
        let widget = Text {
            text_alignment: ctx.get("text-align"),
            squash: ctx.get("squash"),
            style: ctx.style(),
            layout: TextLayout::new(Size::ZERO, false, word_wrap.value_or_default()),
            text: ctx.text.take(),
            word_wrap,
        };

        Ok(Box::new(widget))
    }
}

pub(crate) struct SpanFactory;

impl WidgetFactory for SpanFactory {
    fn make(&self, mut ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let widget = TextSpan {
            text: ctx.text.take(),
            style: ctx.style(),
        };

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_values::ValueExpr;
    use anathema_widget_core::testing::{expression, FakeTerm};

    use crate::testing::test_widget;

    #[test]
    fn word_wrap_excessive_space() {
        test_widget(
            expression("text", Some("hello      how are     you".into()), [], []),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello      how  ║
            ║are     you     ║
            ║                ║
            ║                ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn word_wrap() {
        test_widget(
            expression("text", Some("hello how are you".into()), [], []),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello how are   ║
            ║you             ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn no_word_wrap() {
        test_widget(
            expression(
                "text",
                Some("hello how are you".into()),
                [("wrap".into(), ValueExpr::from("overflow"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hello how are yo║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn break_word_wrap() {
        test_widget(
            expression(
                "text",
                Some("hellohowareyoudoing".into()),
                [("wrap".into(), ValueExpr::from("break"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║hellohowareyoudo║
            ║ing             ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn char_wrap_layout_multiple_spans() {
        let body = [
            expression("span", Some("two".into()), [], []),
            expression("span", Some(" averylongword".into()), [], []),
            expression("span", Some(" bunny ".into()), [], []),
        ];

        let text = expression("text", Some("one".into()), [], body);

        test_widget(
            text,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═════╗
            ║onetwo             ║
            ║averylongword bunny║
            ║                   ║
            ╚═══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn right_alignment() {
        test_widget(
            expression(
                "text",
                Some("a one xxxxxxxxxxxxxxxxxx".into()),
                [("text-align".into(), ValueExpr::from("right"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║            a one ║
            ║xxxxxxxxxxxxxxxxxx║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn centre_alignment() {
        test_widget(
            expression(
                "text",
                Some("a one xxxxxxxxxxxxxxxxxx".into()),
                [("text-align".into(), ValueExpr::from("center"))],
                [],
            ),
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║      a one       ║
            ║xxxxxxxxxxxxxxxxxx║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }
}
