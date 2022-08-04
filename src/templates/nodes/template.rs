use std::cmp::Ordering;

use crate::widgets::{Attribute, Attributes};

use crate::templates::error::Result;
use crate::templates::parser::{Parser, Text};

#[derive(Debug, Clone)]
pub(crate) struct TemplateNode<'src> {
    pub(crate) ident: &'src str,
    pub(crate) text: Option<Text>,
    pub(crate) attributes: Attributes,
    pub(crate) children: Vec<TemplateNode<'src>>,
}

impl<'src> TemplateNode<'src> {
    pub(crate) fn new(ident: &'src str, attributes: Vec<Attribute<'src>>, mut text: Option<Text>) -> Self {
        let attributes = Attributes::from(attributes);

        // If this is a text node then take the text
        // and move it into a span that is injected into the node
        // as a child.
        let children = (ident == "text")
            .then_some(())
            .and_then(|()| text.take())
            .map(|text| vec![Self::span(text, attributes.take_style())])
            .unwrap_or_default();

        Self { ident, attributes, children, text }
    }

    fn span(text: Text, attributes: Attributes) -> Self {
        Self { ident: "span", text: Some(text), attributes, children: vec![] }
    }

    fn add_child(&mut self, child: TemplateNode<'src>) {
        self.children.push(child);
    }
}

// -----------------------------------------------------------------------------
//     - Create a node tree -
//
//     Collapse all nodes into a tree
// -----------------------------------------------------------------------------
pub(crate) fn create_tree(nodes: Parser<'_>) -> Result<Vec<TemplateNode<'_>>> {
    let mut stack: Vec<(usize, TemplateNode)> = vec![];

    for node in nodes {
        let (indent, node) = node?;

        if stack.last().is_none() {
            stack.push((indent, node));
            continue;
        }

        let last_indent = stack.last().map(|(indent, _)| indent).expect("guaranteed to have a node");

        match indent.cmp(last_indent) {
            // Indent is larger than the last node, this means it's
            // a child of the last node
            Ordering::Greater => stack.push((indent, node)),
            // Same level as the last node, making this a sibling
            Ordering::Equal => {
                let (prev_indent, prev) = stack.pop().expect("there is always a node on the stack");
                match stack.last_mut() {
                    Some((parent_indent, parent)) if *parent_indent < prev_indent => parent.add_child(prev),
                    Some((parent_indent, _)) if *parent_indent == prev_indent => stack.push((prev_indent, prev)),
                    Some(_) => unreachable!(),
                    None => stack.push((indent, prev)),
                }
                stack.push((indent, node));
            }
            // This is a parent or possibly grand parent of the node.
            Ordering::Less => {
                let (_, prev) = stack.pop().expect("stack is never empty");
                if let Some((_, parent)) = stack.last_mut() {
                    parent.add_child(prev);
                }

                loop {
                    let (last_indent, last) = stack.pop().unwrap();
                    if stack.is_empty() {
                        stack.push((last_indent, last));
                        break;
                    }
                    if last_indent >= indent {
                        if let Some((_, parent)) = stack.last_mut() {
                            parent.add_child(last);
                        }
                    } else {
                        stack.push((last_indent, last));
                        break;
                    }
                }

                stack.push((indent, node));
            }
        }
    }

    loop {
        match stack.last() {
            Some((last_indent, _)) if *last_indent > 0 => {
                let (_, last) = stack.pop().expect("since there is at least one node this wont fail");
                if let Some((_, parent)) = stack.last_mut() {
                    parent.add_child(last);
                }
            }
            _ => break,
        }
    }

    Ok(stack.into_iter().map(|(_, node)| node).collect::<Vec<_>>())
}
