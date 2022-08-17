use std::collections::HashMap;

use crate::widgets::{HorzEdge, VertEdge};

use super::error::{Error, Result};
use super::nodes::{Kind, Node};

use crate::widgets::{
    fields, Align, Alignment, Animation, Border, Canvas, Expand, HStack, Position, Spacer, Text, TextSpan, VStack,
    Value, Viewport, Widget, WidgetContainer, ZStack,
};

const RESERVED_NAMES: &[&str] = &["if", "for", "else"];

/// `WidgetLookup` contains functions for producing widgets based on the `Node`s ident.
pub struct WidgetLookup {
    inner: HashMap<&'static str, &'static Factory>,
}

type Factory = dyn Fn(&Node, &WidgetLookup) -> Result<WidgetContainer> + Send + Sync + 'static;

impl WidgetLookup {
    pub fn register(&mut self, ident: &'static str, factory: &'static Factory) {
        if self.inner.contains_key(ident) {
            panic!("a widget is already registered with the key \"{ident}\"");
        }

        if RESERVED_NAMES.contains(&ident) {
            panic!("\"{ident}\" is a reserved name");
        }

        self.inner.insert(ident, factory);
    }

    pub(crate) fn make(&self, node: &Node) -> Result<WidgetContainer> {
        let ident = node.ident();
        let f = self.inner.get(ident).ok_or_else(|| Error::UnregisteredWidget(ident.to_string()))?;

        let mut widget = f(node, self)?;

        node.attributes.padding_all().map(|padding| widget.padding = padding);
        widget.background = node.attributes.background();

        let transitions = node.attributes.transitions();

        for (k, value, duration, easing) in transitions {
            let mut animation = Animation::new(duration, easing);
            animation.set_src(value);
            widget.animation.push(k, animation);
        }

        Ok(widget)
    }
}

// -----------------------------------------------------------------------------
//     - Text -
// -----------------------------------------------------------------------------
pub fn text_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;

    let mut widget = Text::new();
    widget.trim_start = attribs.trim_start();
    widget.trim_end = attribs.trim_end();
    widget.collapse_spaces = attribs.collapse_spaces();
    widget.word_wrap = attribs.word_wrap();
    widget.text_alignment = attribs.text_alignment();

    for child in &node.children {
        let child = lookup.make(child)?;
        widget.spans.push(child);
    }

    Ok(widget.into_container(node.id()))
}

pub fn span_widget(node: &Node, _: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;

    let text = match &node.kind {
        Kind::Span(text) => text,
        Kind::Node { .. } => return Err(Error::InvalidTextWidget),
    };

    let mut widget = TextSpan::new(text);
    widget.style = attribs.style();

    Ok(widget.into_container(node.id()))
}

impl Default for WidgetLookup {
    fn default() -> Self {
        let mut inst = Self { inner: HashMap::new() };

        inst.register("alignment", &alignment_widget);
        inst.register("border", &border_widget);
        inst.register("canvas", &canvas_widget);
        inst.register("expand", &expand_widget);
        inst.register("hstack", &hstack_widget);
        inst.register("position", &position_widget);
        inst.register("spacer", &spacer_widget);
        inst.register("text", &text_widget);
        inst.register("span", &span_widget);
        inst.register("viewport", &viewport_widget);
        inst.register("vstack", &vstack_widget);
        inst.register("zstack", &zstack_widget);

        inst
    }
}

// -----------------------------------------------------------------------------
//     - Alignment -
// -----------------------------------------------------------------------------
fn alignment_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;
    let (align, duration_easing) = match attribs.get_value(fields::ALIGNMENT) {
        Some(Value::Alignment(align)) => (align, None),
        Some(Value::Transition(value, duration, easing)) => match value.as_ref() {
            Value::Alignment(ref align) => (*align, Some((duration, easing))),
            _ => (Align::TopLeft, Some((duration, easing))),
        },
        _ => (Align::TopLeft, None),
    };

    let mut alignment = Alignment::new(align);
    if let Some(child) = node.children.first() {
        let mut child = lookup.make(child)?;
        if let Some((duration, easing)) = duration_easing {
            child.animation.set_position(duration, easing);
        }
        alignment.add_child(child);
    }
    Ok(alignment.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - ZStack -
// -----------------------------------------------------------------------------
fn zstack_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let mut widget = ZStack::new(node.attributes.width(), node.attributes.height());
    widget.min_width = node.attributes.min_width();
    widget.min_height = node.attributes.min_height();

    for child in &node.children {
        let child = lookup.make(child)?;
        widget.children.push(child);
    }

    Ok(widget.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - HStack -
// -----------------------------------------------------------------------------
fn hstack_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let mut widget = HStack::new(node.attributes.width(), node.attributes.height());
    widget.min_width = node.attributes.min_width();
    widget.min_height = node.attributes.min_height();

    for child in &node.children {
        let child = lookup.make(child)?;
        widget.children.push(child);
    }

    Ok(widget.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - VStack -
// -----------------------------------------------------------------------------
fn vstack_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;

    let mut widget = VStack::new(attribs.width(), attribs.height());
    widget.min_width = attribs.min_width();
    widget.min_height = attribs.min_height();

    for child in &node.children {
        let child = lookup.make(child)?;
        widget.children.push(child);
    }

    Ok(widget.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - Spacer -
// -----------------------------------------------------------------------------
fn spacer_widget(node: &Node, _lookup: &WidgetLookup) -> Result<WidgetContainer> {
    Ok(Spacer.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - Position -
// -----------------------------------------------------------------------------
fn position_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;

    let horz_edge = match attribs.left() {
        Some(left) => HorzEdge::Left(left),
        None => match attribs.right() {
            Some(right) => HorzEdge::Right(right),
            None => HorzEdge::Left(0),
        },
    };

    let vert_edge = match attribs.top() {
        Some(top) => VertEdge::Top(top),
        None => match attribs.bottom() {
            Some(bottom) => VertEdge::Bottom(bottom),
            None => VertEdge::Top(0),
        },
    };

    let mut widget = Position::new(horz_edge, vert_edge);
    if let Some(child) = node.children.first() {
        widget.child = Some(lookup.make(child)?);
    }
    Ok(widget.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - Border -
// -----------------------------------------------------------------------------
fn border_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;
    let border_style = attribs.border_style();
    let sides = attribs.sides();
    let width = attribs.width();
    let height = attribs.height();

    let mut widget = Border::new(border_style, sides, width, height);
    widget.min_width = attribs.min_width();
    widget.min_height = attribs.min_height();
    widget.style = attribs.style();
    if let Some(child) = node.children.first() {
        widget.child = Some(lookup.make(child)?);
    }
    Ok(widget.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - Canvas -
// -----------------------------------------------------------------------------
fn canvas_widget(node: &Node, _: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;
    let widget = Canvas::new(attribs.width(), attribs.height());
    Ok(widget.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - Expand -
// -----------------------------------------------------------------------------
fn expand_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let direction = node.attributes.direction();

    let mut widget = Expand::new(node.attributes.factor(), direction);
    widget.style = node.attributes.style();
    if let Some(fill) = node.attributes.fill() {
        widget.fill = fill.to_string();
    }
    if let Some(child) = node.children.first() {
        let child = lookup.make(child)?;
        widget.child = Some(child);
    }

    let widget = widget.into_container(node.id());
    Ok(widget)
}

fn viewport_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let offset = node.attributes.offset();
    let mut widget = Viewport::new(offset.unwrap_or_default());

    for child in &node.children {
        let child = lookup.make(child)?;
        widget.children.push(child);
    }

    Ok(widget.into_container(node.id()))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::{fields, Attributes, BorderStyle, NodeId};

    fn node_to_widget(node: &Node) -> WidgetContainer {
        let lookup = WidgetLookup::default();
        lookup.make(node).unwrap()
    }

    #[test]
    fn lookup_border() {
        let mut attributes = Attributes::empty();
        attributes.set(fields::MIN_WIDTH, 10u64);
        attributes.set(fields::MIN_HEIGHT, 3u64);
        attributes.set(fields::BORDER_STYLE, BorderStyle::Custom("01234567".into()));
        let node =
            Node { kind: Kind::Node { ident: "border".into() }, children: vec![], id: NodeId::auto(), attributes };

        let mut widget = node_to_widget(&node);
        let border = widget.to::<Border>();
        assert_eq!(Some(10), border.min_width);
        assert_eq!(Some(3), border.min_height);
        assert_eq!(['0', '1', '2', '3', '4', '5', '6', '7'], border.edges);
    }

    #[test]
    fn lookup_vstack() {
        let mut attributes = Attributes::empty();
        attributes.set(fields::MIN_WIDTH, 10u64);
        attributes.set(fields::MIN_HEIGHT, 3u64);
        let node =
            Node { kind: Kind::Node { ident: "vstack".into() }, children: vec![], id: NodeId::auto(), attributes };

        let mut widget = node_to_widget(&node);
        let stack = widget.to::<VStack>();
        assert_eq!(Some(10), stack.min_width);
        assert_eq!(Some(3), stack.min_height);
    }

    #[test]
    fn lookup_hstack() {
        let mut attributes = Attributes::empty();
        attributes.set(fields::MIN_WIDTH, 10u64);
        attributes.set(fields::MIN_HEIGHT, 3u64);
        let node =
            Node { kind: Kind::Node { ident: "hstack".into() }, children: vec![], id: NodeId::auto(), attributes };

        let mut widget = node_to_widget(&node);
        let stack = widget.to::<HStack>();
        assert_eq!(Some(10), stack.min_width);
        assert_eq!(Some(3), stack.min_height);
    }

    #[test]
    fn lookup_zstack() {
        let mut attributes = Attributes::empty();
        attributes.set(fields::MIN_WIDTH, 10u64);
        attributes.set(fields::MIN_HEIGHT, 3u64);
        let node =
            Node { kind: Kind::Node { ident: "zstack".into() }, children: vec![], id: NodeId::auto(), attributes };

        let mut widget = node_to_widget(&node);
        let stack = widget.to::<ZStack>();
        assert_eq!(Some(10), stack.min_width);
        assert_eq!(Some(3), stack.min_height);
    }
}
