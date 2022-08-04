pub mod error;

pub use error::Result;

mod ctx;
mod lookup;
mod nodes;
mod parser;

pub use ctx::{DataCtx, IncludeCache, NodeCtx, SubContext};
pub use lookup::WidgetLookup;
pub use nodes::widget::WidgetNode;
pub use nodes::{diff, Node};

// Src -> WidgetNodes -> Nodes -> Widgets
// WidgetNodes -> Nodes -> Diff -> Widgets

pub fn parse(src: &str) -> Result<Vec<WidgetNode>> {
    let lexer = parser::lexer::Lexer::new(src);
    let parser = parser::Parser::new(lexer);
    let node_tree = nodes::template::create_tree(parser)?;
    nodes::widget::to_widget_nodes(node_tree, false)
}

pub fn to_nodes(
    widget_nodes: &[WidgetNode],
    data_ctx: &SubContext<'_>,
    node_ctx: &mut NodeCtx<'_>,
) -> Result<Vec<Node>> {
    let mut nodes = vec![];
    for widget_node in widget_nodes {
        nodes.extend(nodes::to_nodes(widget_node, data_ctx, node_ctx)?);
    }
    Ok(nodes)
}

pub fn build_widget_tree(
    lookup: &WidgetLookup,
    widget_nodes: &[WidgetNode],
    data_ctx: &SubContext<'_>,
    node_ctx: &mut NodeCtx<'_>,
) -> Result<Vec<crate::widgets::WidgetContainer>> {
    let mut widgets = vec![];
    let nodes = to_nodes(widget_nodes, data_ctx, node_ctx)?;

    for node in &nodes {
        let widget = lookup.make(node)?;
        widgets.push(widget);
    }

    Ok(widgets)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::testing::test_widget_container;

    fn test_parse_input(src: &str, expected: &str) {
        let lookup = WidgetLookup::default();
        let output = parse(src).unwrap();
        let ctx = DataCtx::empty();
        let sub = SubContext::new(&ctx);
        let mut include_cache = IncludeCache::default();
        let mut node_ctx = NodeCtx::new(&mut include_cache);
        let mut tree = build_widget_tree(&lookup, &output, &sub, &mut node_ctx).unwrap();
        let root = tree.remove(0);
        test_widget_container(root, expected);
    }

    #[test]
    fn parse_valid_src() {
        let input = r#"
        border:
            vstack [background: red]:
                text: "A hot cuppa "
                    span: "tea"
                text: "A cold ice cream"
                    span: " later"
        "#;

        test_parse_input(
            input,
            r#"
            ┌──────────────────────┐
            │A hot cuppa tea       │
            │A cold ice cream later│
            └──────────────────────┘
            "#,
        );
    }
}
