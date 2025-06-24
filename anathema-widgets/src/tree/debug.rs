use std::fmt::{Result, Write};

use anathema_store::tree::{Node, TreeValues};

use crate::{WidgetContainer, WidgetId, WidgetKind, WidgetTreeView};

pub fn debug_tree(tree: &WidgetTreeView<'_, '_>) -> String {
    let mut output = String::new();

    let (nodes, values) = tree.nodes_and_values();

    walk_tree(0, nodes, values, &mut output).unwrap();
    output
}

fn walk_tree(level: usize, nodes: &[Node], values: &TreeValues<WidgetContainer<'_>>, output: &mut String) -> Result {
    for node in nodes {
        let Some((_, widget)) = values.get(node.value()) else { continue };
        write_value(widget, level + 1, node.value(), output)?;
        walk_tree(level + 1, node.children(), values, output)?;
    }

    Ok(())
}

fn write_value(widget: &WidgetContainer<'_>, level: usize, widget_id: WidgetId, output: &mut String) -> Result {
    write!(output, "{}:{}", widget_id.index(), widget_id.generation())?;
    let indent = " ".repeat(level * 4);
    write!(output, "{indent}{}:{} ", widget_id.index(), widget_id.generation())?;

    match &widget.kind {
        WidgetKind::Element(element) => writeln!(output, "{}", element.ident),
        WidgetKind::For(_) => writeln!(output, "<for>"),
        WidgetKind::With(_) => writeln!(output, "<with>"),
        WidgetKind::Iteration(_) => writeln!(output, "<iter>"),
        WidgetKind::ControlFlow(_) => writeln!(output, "<control flow>"),
        WidgetKind::ControlFlowContainer(_) => writeln!(output, "<control flow container>"),
        WidgetKind::Component(_) => writeln!(output, "<component>"),
        WidgetKind::Slot => writeln!(output, "<slot>"),
    }
}
