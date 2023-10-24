use anathema_render::{Style, Color};
use anathema_values::{Attributes, Context, NodeId, ValueExpr};

pub struct FactoryContext<'a> {
    pub ident: &'a str,
    pub attributes: &'a Attributes,
    pub ctx: &'a Context<'a, 'a>,
    pub node_id: NodeId,
    pub text: Option<&'a ValueExpr>,
}

impl<'a> FactoryContext<'a> {
    pub fn new(
        ctx: &'a Context<'a, 'a>,
        node_id: NodeId,
        ident: &'a str,
        attributes: &'a Attributes,
        text: Option<&'a ValueExpr>,
    ) -> Self {
        Self {
            ctx,
            ident,
            node_id,
            attributes,
            text,
        }
    }

    fn node_id(&self) -> Option<&NodeId> {
        Some(&self.node_id)
    }

    pub fn text(&self) -> String {
        self.text
            .and_then(|value| value.eval_string(self.ctx, self.node_id()))
            .unwrap_or_else(String::new)
    }

    pub fn style(&self) -> Style {
        let mut style = Style::new();

        style.fg = self.get_color("foreground");
        style.set_bold(self.is_true("bold"));
        style.set_italic(self.is_true("italic"));
        style.set_dim(self.is_true("dim"));
        style.set_underlined(self.is_true("underline"));
        style.set_overlined(self.is_true("overline"));
        style.set_crossed_out(self.is_true("crossed-out"));
        style.set_inverse(self.is_true("inverse"));

        style
    }

    pub fn is_true(&self, name: &str) -> bool {
        self.ctx.attribute(name, self.node_id(), self.attributes).unwrap_or(false)
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.ctx.attribute(name, self.node_id(), self.attributes)
    }

    pub fn get_color(&self, name: &str) -> Option<Color> {
        self.ctx.attribute(name, self.node_id(), self.attributes)
    }

    pub fn get_usize(&self, name: &str) -> Option<usize> {
        self.ctx.attribute(name, self.node_id(), self.attributes)
    }
}
