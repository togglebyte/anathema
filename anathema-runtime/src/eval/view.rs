use crate::{ElementId, elements::{Element, Elements}};

pub struct View {
    node_id: ElementId,
    children: Vec<ElementId>,
}

pub fn eval_view<'bp>(parent: Option<ElementId>, element: ElementId, elements: &Elements<'bp>) {
    let parent = match parent {
        None => elements.root,
        Some(parent) => parent,
    };
}
