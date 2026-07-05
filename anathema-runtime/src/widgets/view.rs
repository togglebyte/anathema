use crate::ElementId;
use crate::elements::{Element, Elements};

#[derive(Debug)]
pub struct View {
    element: ElementId,
    children: Vec<View>,
}

#[derive(Debug, Copy, Clone)]
enum ControlFlow {
    Unresolved,
    Resolved,
}

impl ControlFlow {
    fn unresolved(self) -> bool {
        match self {
            Self::Unresolved => true,
            Self::Resolved => false,
        }
    }
}

pub fn build_view_tree<'bp>(element: ElementId, elements: &Elements<'bp>) -> View {
    let mut children = vec![];

    let node = &elements[element];
    for child in &node.children {
        add_child(&mut children, *child, elements, ControlFlow::Unresolved);
    }

    View { element, children }
}

fn add_child<'bp>(views: &mut Vec<View>, element: ElementId, elements: &Elements<'bp>, mut control_flow: ControlFlow) {
    let node = &elements[element];
    let s = format!("{:?}", node.element);
    eprintln!("{s}");
    match &node.element {
        Element::ControlFlow => add_child(views, element, elements, ControlFlow::Unresolved),
        Element::Condition(Some(cond)) if cond.truthiness() && control_flow.unresolved() => {
            control_flow = ControlFlow::Resolved;
            for element in &node.children {
                add_child(views, *element, elements, control_flow);
            }
        }
        Element::Condition(None) if control_flow.unresolved() => {
            control_flow = ControlFlow::Resolved;
            for element in &node.children {
                add_child(views, *element, elements, control_flow);
            }
        }
        Element::Condition(_) => return,
        Element::Widget(ref_cell) => {
            let mut children = vec![];

            let node = &elements[element];
            for child in &node.children {
                add_child(&mut children, *child, elements, control_flow);
            }

            let view = View { element, children };
            views.push(view);
        }
        _ => {
            for element in &node.children {
                add_child(views, *element, elements, control_flow);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::RunBuilder;

    #[test]
    fn add_children() {
        let tpl = "
            if true
                with val as 'hello'
                    node val
            else
                with val as 'not hello'
                    node val
        ";

        let mut test = RunBuilder::from_src(tpl);
        let mut inst = test.finish();
        inst.eval(|ctx| {
            let view = build_view_tree(ctx.elements.root, ctx.elements);
            eprintln!("{view:#?}");
            eprintln!("{:?}", ctx.elements);
            panic!();
        });

        // assert_eq!(expected, actual);
    }
}
