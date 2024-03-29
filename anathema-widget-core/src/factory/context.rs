use anathema_values::{Attributes, Context, DynValue, NodeId, Value};

use crate::WidgetStyle;

pub struct FactoryContext<'a> {
    pub ident: &'a str,
    pub attributes: &'a Attributes,
    pub ctx: &'a Context<'a, 'a>,
    pub node_id: NodeId,
    pub text: Value<String>,
}

impl<'a> FactoryContext<'a> {
    pub fn new(
        ctx: &'a Context<'a, 'a>,
        node_id: NodeId,
        ident: &'a str,
        attributes: &'a Attributes,
        text: Value<String>,
    ) -> Self {
        Self {
            ctx,
            ident,
            node_id,
            attributes,
            text,
        }
    }

    pub fn node_id(&self) -> Option<&NodeId> {
        Some(&self.node_id)
    }

    pub fn style(&self) -> WidgetStyle {
        WidgetStyle {
            fg: self.get("foreground"),
            bg: self.get("background"),
            bold: self.get("bold"),
            dim: self.get("dim"),
            italic: self.get("italic"),
            underlined: self.get("underlined"),
            crossed_out: self.get("crossed-out"),
            overlined: self.get("overlined"),
            inverse: self.get("inverse"),
        }
    }

    pub fn get<T: DynValue>(&self, name: &str) -> Value<T> {
        let Some(val) = self.attributes.get(name) else {
            return Value::Empty;
        };
        T::init_value(self.ctx, &self.node_id, val)
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::TestState;
    use anathema_values::ValueExpr;

    use super::*;

    #[test]
    fn get_attribute() {
        let state = TestState::new();
        let ctx = Context::root(&state);
        let mut attributes = Attributes::new();
        attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

        let ctx = FactoryContext::new(&ctx, 0.into(), "border", &attributes, Value::Empty);

        let name = ctx.get::<String>("name");
        assert_eq!("Dirk Gently", name.str());
    }
}
