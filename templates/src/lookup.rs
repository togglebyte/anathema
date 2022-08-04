use std::collections::HashMap;

use widgets::{HorzEdge, VertEdge};

use crate::error::{Error, Result};
use crate::nodes::{Kind, Node};

use widgets::{
    fields, Align, Alignment, Animation, Axis, Border, Canvas, Expand, HStack, 
    Position, ScrollView, Spacer, Text, TextSpan, VStack, Value, Widget, WidgetContainer, ZStack,
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
        let f = self
            .inner
            .get(ident)
            .ok_or_else(|| Error::UnregisteredWidget(ident.to_string()))?;

        let mut widget = f(node, self)?;

        node.attributes
            .padding_all()
            .map(|padding| widget.padding = padding);
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
pub fn text_widget(node: &Node, _: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;

    let mut widget = Text::new();
    widget.trim_start = attribs.trim_start();
    widget.trim_end = attribs.trim_end();
    widget.collapse_spaces = attribs.collapse_spaces();
    widget.word_wrap = attribs.word_wrap();
    widget.text_alignment = attribs.text_alignment();

    // All the spans
    for text_child in &node.children {
        let attribs = &text_child.attributes;

        let text = match &text_child.kind {
            Kind::Span(text) => text,
            Kind::Node { .. } => return Err(Error::InvalidTextWidget),
        };

        let mut span = TextSpan::new(text);
        span.style = attribs.style();
        widget.add_span(span);
    }

    Ok(widget.into_container(node.id()))
}

impl Default for WidgetLookup {
    fn default() -> Self {
        let mut inst = Self {
            inner: HashMap::new(),
        };

        inst.register("alignment", &alignment_widget);
        inst.register("border", &border_widget);
        inst.register("canvas", &canvas_widget);
        inst.register("expand", &expand_widget);
        inst.register("position", &position_widget);
        inst.register("spacer", &spacer_widget);
        inst.register("text", &text_widget);
        inst.register("scrollview", &scrollview_widget);
        inst.register("vstack", &vstack_widget);
        inst.register("hstack", &hstack_widget);
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
//     - Scroll view -
// -----------------------------------------------------------------------------
fn scrollview_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let attribs = &node.attributes;

    let mut scrollview = ScrollView::new(
        attribs.max_children(),
        attribs.axis().unwrap_or(Axis::Vertical),
        attribs.offset(),
        attribs.auto_scroll(),
        attribs.reverse(),
    );

    for child in &node.children {
        let child = lookup.make(child)?;
        scrollview.add_child(child);
    }

    Ok(scrollview.into_container(node.id()))
}

// -----------------------------------------------------------------------------
//     - ZStack -
// -----------------------------------------------------------------------------
fn zstack_widget(node: &Node, lookup: &WidgetLookup) -> Result<WidgetContainer> {
    let mut widget = ZStack::new(node.attributes.width(), node.attributes.height());

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
