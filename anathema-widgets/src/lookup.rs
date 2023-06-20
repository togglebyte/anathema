
use std::collections::HashMap;

use crate::alignment::AlignmentFactory;
use crate::border::BorderFactory;

use crate::error::{Error, Result};
use crate::expand::ExpandFactory;
use crate::gen::store::Store;
use crate::hstack::HStackFactory;
use crate::position::PositionFactory;
use crate::spacer::SpacerFactory;
use crate::template::Template;
use crate::text::{SpanFactory, TextFactory};
use crate::values::ValuesAttributes;
use crate::viewport::ViewportFactory;
use crate::vstack::VStackFactory;
use crate::widget::AnyWidget;
use crate::zstack::ZStackFactory;
use crate::{Padding, TextPath, WidgetContainer};

const RESERVED_NAMES: &[&str] = &["if", "for", "else"];

pub trait WidgetFactory {
    fn make(
        &self,
        store: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>>;
}

pub struct Lookup(HashMap<String, Box<dyn WidgetFactory>>);

impl Lookup {
    pub fn exec<'tpl, 'parent>(
        &self,
        template: &'tpl Template,
        values: &Store<'parent>,
    ) -> Result<WidgetContainer<'tpl>> {
        match &template {
            Template::Node {
                ident,
                attributes,
                text,
                children,
            } => {
                let factory = self
                    .0
                    .get(ident)
                    .ok_or_else(|| Error::UnregisteredWidget(ident.to_string()))?;
                let values = ValuesAttributes::new(values, attributes);
                let background = values.background();
                let padding = values.padding_all().unwrap_or_else(|| Padding::ZERO);
                let widget = factory.make(values, text.as_ref())?;
                let mut container = WidgetContainer::new(widget, children);
                container.background = background;
                container.padding = padding;
                Ok(container)
            }
            _ => panic!("there should only ever be nodes here, not {:?}", template),
        }
    }

    pub fn register(&mut self, ident: impl Into<String>, factory: Box<dyn WidgetFactory>) -> Result<()> {
        let ident = ident.into();
        if RESERVED_NAMES.contains(&ident.as_str()) {
            return Err(Error::ReservedName(ident));
        }

        if self.0.contains_key(&ident) {
            return Err(Error::ExistingName(ident));
        }

        self.0.insert(ident, factory);

        Ok(())
    }
}

impl Default for Lookup {
    fn default() -> Self {
        let mut inner = HashMap::<_, Box<dyn WidgetFactory>>::new();
        inner.insert("alignment".to_string(), Box::new(AlignmentFactory));
        inner.insert("border".to_string(), Box::new(BorderFactory));
        inner.insert("expand".to_string(), Box::new(ExpandFactory));
        inner.insert("hstack".to_string(), Box::new(HStackFactory));
        inner.insert("position".to_string(), Box::new(PositionFactory));
        inner.insert("spacer".to_string(), Box::new(SpacerFactory));
        inner.insert("span".to_string(), Box::new(SpanFactory));
        inner.insert("text".to_string(), Box::new(TextFactory));
        inner.insert("viewport".to_string(), Box::new(ViewportFactory));
        inner.insert("vstack".to_string(), Box::new(VStackFactory));
        inner.insert("zstack".to_string(), Box::new(ZStackFactory));
        Self(inner)
    }
}

// // // -----------------------------------------------------------------------------
// // //     - Canvas -
// // // -----------------------------------------------------------------------------
// // fn canvas_widget<'gen, 'ctx>(
// //     node: &'gen WidgetTemplate,
// //     _: &WidgetLookup,
// // ) -> Result<WidgetContainer<'gen>> {
// //     panic!()
// //     // let attribs = &node.attributes;
// //     // let widget = Canvas::new(attribs.width(), attribs.height());
// //     // Ok(widget.into_container(node.id()))
// // }

// fn item_widget(_: ValueLookup<'_>) -> Result<Box<dyn AnyWidget>> {
//     Ok(Box::new(Item))
// }

// #[cfg(test)]
// mod test {
//     // use anathema_widgets::{fields, Attributes, BorderStyle, NodeId};

//     // use super::*;

//     // fn node_to_widget(kind: &WidgetKind, attribs: &Attributes) -> WidgetContainer {
//     //     let lookup = WidgetLookup::default();
//     //     lookup.make(kind, attribs).unwrap()
//     // }

//     // #[test]
//     // fn lookup_border() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     attributes.set(fields::BORDER_STYLE, BorderStyle::Custom("01234567".into()));
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("border".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let border = widget.to_mut::<Border>();
//     //     assert_eq!(Some(10), border.min_width);
//     //     assert_eq!(Some(3), border.min_height);
//     //     assert_eq!(['0', '1', '2', '3', '4', '5', '6', '7'], border.edges);
//     // }

//     // #[test]
//     // fn lookup_vstack() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("vstack".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let stack = widget.to_mut::<VStack>();
//     //     assert_eq!(Some(10), stack.min_width);
//     //     assert_eq!(Some(3), stack.min_height);
//     // }

//     // #[test]
//     // fn lookup_hstack() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("hstack".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let stack = widget.to_mut::<HStack>();
//     //     assert_eq!(Some(10), stack.min_width);
//     //     assert_eq!(Some(3), stack.min_height);
//     // }

//     // #[test]
//     // fn lookup_zstack() {
//     //     let mut attributes = Attributes::empty();
//     //     attributes.set(fields::MIN_WIDTH, 10u64);
//     //     attributes.set(fields::MIN_HEIGHT, 3u64);
//     //     let node = WidgetTemplate {
//     //         kind: TemplateKind::Node(WidgetKind::Node("zstack".into()), attributes),
//     //         children: vec![],
//     //         id: NodeId::empty(),
//     //     };

//     //     let mut widget = node_to_widget(&node);
//     //     let stack = widget.to_mut::<ZStack>();
//     //     assert_eq!(Some(10), stack.min_width);
//     //     assert_eq!(Some(3), stack.min_height);
//     // }
// }
