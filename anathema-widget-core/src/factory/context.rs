use anathema_render::{Color, Style};
use anathema_values::{Attributes, Context, NodeId, Path, Value, ValueExpr};

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

    pub fn text(&self) -> Value<String> {
        let string = self
            .text
            .and_then(|value| value.eval_string(self.ctx, self.node_id()))
            .unwrap_or_else(String::new);
        Value::Static(string)
    }

    pub fn magic(&self) {
        let x = self.text.unwrap();
        let y = x;
        // let ValueExpr::List(list) = self.text.unwrap() else { panic!() };

        // let list = list.to_vec();

        // for x in list {
        //     if let Some(y) = x.eval_path(self.ctx, self.node_id()) {
        //         let lala = y;
        //         let scope_val = self.ctx.scope.lookup(&lala);

        //         let end = scope_val;
        //     }

        // }

        // //     // .and_then(|value| value.eval_string(self.ctx, self.node_id()))
    }

    pub fn style(&self) -> Style {
        panic!()
        // let mut style = Style::new();

        // style.fg = self.get_color("foreground");
        // style.set_bold(self.is_true("bold"));
        // style.set_italic(self.is_true("italic"));
        // style.set_dim(self.is_true("dim"));
        // style.set_underlined(self.is_true("underline"));
        // style.set_overlined(self.is_true("overline"));
        // style.set_crossed_out(self.is_true("crossed-out"));
        // style.set_inverse(self.is_true("inverse"));

        // style
    }

    pub fn is_true(&self, name: &str) -> bool {
        panic!()
        // self.ctx
        //     .attribute(name, self.node_id(), self.attributes)
        //     .unwrap_or(false)
    }

    pub fn get_bool(&self, name: &str) -> Value<bool> {
        self.ctx.attribute(name, self.node_id(), self.attributes)
    }

    pub fn get_color(&self, name: &str) -> Value<Color> {
        self.ctx.attribute(name, self.node_id(), self.attributes)
    }

    pub fn get_usize(&self, name: &str) -> Value<usize> {
        self.ctx.attribute(name, self.node_id(), self.attributes)
    }
}
