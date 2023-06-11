use std::borrow::Cow;
use std::collections::HashMap;

use crate::border::BorderFactory;
use crate::contexts::DataCtx;
use crate::error::{Error, Result};
use crate::gen::store::Store;
use crate::template::Template;
use crate::text::{TextFactory, SpanFactory};
use crate::values::{Layout, ValuesAttributes};
use crate::viewport::ViewportFactory;
use crate::widget::AnyWidget;
use crate::{Attributes, TextPath, Value, Widget, WidgetContainer};

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
                let something = ValuesAttributes::new(values, attributes);
                let widget = factory.make(something, text.as_ref())?;
                Ok(WidgetContainer::new(widget, children))
            }
            _ => panic!("there should only ever be nodes here, not {:?}", template),
        }
    }
}

impl Default for Lookup {
    fn default() -> Self {
        let mut inner = HashMap::<_, Box<dyn WidgetFactory>>::new();
        inner.insert("border".to_string(), Box::new(BorderFactory));
        inner.insert("span".to_string(), Box::new(SpanFactory));
        inner.insert("text".to_string(), Box::new(TextFactory));
        inner.insert("viewport".to_string(), Box::new(ViewportFactory));
        Self(inner)
    }
}

// // -----------------------------------------------------------------------------
// //     - Alignment -
// // -----------------------------------------------------------------------------
// // fn alignment_widget<'gen>(
// //     node: &'gen WidgetTemplate,
// //     lookup: &WidgetLookup,
// // ) -> Result<Box<dyn Widget>> {
// //     panic!()
// //     // let (align, duration_easing) = match node.attributes.get_value(fields::ALIGNMENT) {
// //     //     Some(Value::Alignment(align)) => (align, None),
// //     //     Some(Value::Transition(value, duration, easing)) => match value.as_ref() {
// //     //         Value::Alignment(ref align) => (*align, Some((duration, easing))),
// //     //         _ => (Align::TopLeft, Some((duration, easing))),
// //     //     },
// //     //     _ => (Align::TopLeft, None),
// //     // };

// //     // let mut alignment = Alignment::new(align);
// //     // if let Some(child) = node.children.first() {
// //     //     let mut child = lookup.make(child)?;
// //     //     if let Some((duration, easing)) = duration_easing {
// //     //         child.animation.set_position(duration, easing);
// //     //     }
// //     //     alignment.add_child(child);
// //     // }
// //     // Ok(alignment.into_container(node.id()))
// // }

// // // -----------------------------------------------------------------------------
// // //     - ZStack -
// // // -----------------------------------------------------------------------------
// // fn zstack_widget<'gen, 'ctx>(
// //     node: &'gen WidgetTemplate,
// //     lookup: &WidgetLookup,
// // ) -> Result<WidgetContainer<'gen>> {
// //     panic!()
// //     // let mut widget = ZStack::new(node.attributes.width(), node.attributes.height());
// //     // widget.min_width = node.attributes.min_width();
// //     // widget.min_height = node.attributes.min_height();

// //     // for child in &node.children {
// //     //     let child = lookup.make(child)?;
// //     //     widget.children.push(child);
// //     // }

// //     // Ok(widget.into_container(node.id()))
// // }

// // // -----------------------------------------------------------------------------
// // //     - HStack -
// // // -----------------------------------------------------------------------------
// // fn hstack_widget<'gen, 'ctx>(values: ValueLookup<'gen, 'ctx>) -> Result<Box<dyn Widget>> {
// //     let width = values.width();
// //     let height = values.height();
// //     let mut widget = HStack::new(width, height);
// //     widget.min_width = values.min_width();
// //     widget.min_height = values.min_height();
// //     // Ok(Box::new(widget))
// //     panic!()
// // }

// // // -----------------------------------------------------------------------------
// // //     - VStack -
// // // -----------------------------------------------------------------------------
// // fn vstack_widget(values: ValueLookup<'_>) -> Result<Box<dyn AnyWidget>> {
// //     let width = values.width();
// //     let height = values.height();
// //     let mut widget = VStack::new(width, height);
// //     widget.min_width = values.min_width();
// //     widget.min_height = values.min_height();
// //     // Ok(Box::new(widget))
// //     panic!()
// // }

// // // -----------------------------------------------------------------------------
// // //     - Spacer -
// // // -----------------------------------------------------------------------------
// // fn spacer_widget<'gen, 'ctx>(
// //     node: &'gen WidgetTemplate,
// //     _lookup: &WidgetLookup,
// // ) -> Result<WidgetContainer<'gen>> {
// //     panic!()
// //     // Ok(Spacer.into_container(node.id()))
// // }

// // // -----------------------------------------------------------------------------
// // //     - Position -
// // // -----------------------------------------------------------------------------
// // fn position_widget<'gen, 'ctx>(
// //     node: &'gen WidgetTemplate,
// //     lookup: &WidgetLookup,
// // ) -> Result<WidgetContainer<'gen>> {
// //     panic!()
// //     // let attribs = &node.attributes;

// //     // let horz_edge = match attribs.left() {
// //     //     Some(left) => HorzEdge::Left(left),
// //     //     None => match attribs.right() {
// //     //         Some(right) => HorzEdge::Right(right),
// //     //         None => HorzEdge::Left(0),
// //     //     },
// //     // };

// //     // let vert_edge = match attribs.top() {
// //     //     Some(top) => VertEdge::Top(top),
// //     //     None => match attribs.bottom() {
// //     //         Some(bottom) => VertEdge::Bottom(bottom),
// //     //         None => VertEdge::Top(0),
// //     //     },
// //     // };

// //     // let mut widget = Position::new(horz_edge, vert_edge);
// //     // if let Some(child) = node.children.first() {
// //     //     widget.child = Some(lookup.make(child)?);
// //     // }
// //     // Ok(widget.into_container(node.id()))
// // }

// // -----------------------------------------------------------------------------
// //     - Border -
// // -----------------------------------------------------------------------------
// fn border_widget(values: ValueLookup<'_>) -> Result<Box<dyn AnyWidget>> {
//     let border_style = values.border_style();
//     let sides = values.sides();
//     let width = values.width();
//     let height = values.height();

//     let mut widget = Border::new(&*border_style, sides, width, height);
//     widget.min_width = values.min_width();
//     widget.min_height = values.min_height();
//     widget.style = values.style();
//     Ok(Box::new(widget))
// }

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

// // // -----------------------------------------------------------------------------
// // //     - Expand -
// // // -----------------------------------------------------------------------------
// // fn expand_widget<'gen, 'ctx>(values: ValueLookup<'gen, 'ctx>) -> Result<Box<dyn Widget>> {
// //     let direction = values.direction();
// //     let factor = values.factor();
// //     let mut widget = Expand::new(factor, direction);
// //     widget.style = values.style();

// //     if let Some(fill) = values.fill() {
// //         widget.fill = fill.to_string();
// //     }

// //     // Ok(Box::new(widget))
// //     panic!()
// // }

// fn viewport_widget(values: ValueLookup<'_>) -> Result<Box<dyn AnyWidget>> {
//     let data_source = values.get_attrib("source").map(|v| v.to_owned());
//     let binding = values.get_attrib("binding").map(|v| v.to_string());
//     let item = values.get_int("item").unwrap_or(0) as usize;
//     let offset = values.get_signed_int("offset").unwrap_or(0) as isize;
//     let offset = Offset { element: item, cell: offset };
//     let direction = values.direction().unwrap_or(Direction::Forward);
//     let widget = Viewport::new(data_source, binding, offset, direction);
//     Ok(Box::new(widget))
// }

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
