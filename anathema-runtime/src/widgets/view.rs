use crate::ElementId;
use crate::elements::{Element, Elements};

#[derive(Debug)]
pub struct View {
    element: ElementId,
    children: Vec<View>,
}

pub fn build_view_tree<'bp>(element: ElementId, elements: &Elements<'bp>) -> View {
    let mut children = vec![];

    let node = &elements[element];
    for child in &node.children {
        add_child(&mut children, *child, elements);
    }

    View { element, children }
}

fn add_child<'bp>(views: &mut Vec<View>, element: ElementId, elements: &Elements<'bp>) {
    let node = &elements[element];
    match &node.element {
        Element::Widget(ref_cell) => {
            let mut children = vec![];

            let node = &elements[element];
            for child in &node.children {
                add_child(&mut children, *child, elements);
            }

            let view = View { element, children };
            views.push(view);
        }
        _ => {
            for element in &node.children {
                add_child(views, *element, elements);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::testing::RunBuilder;

    use super::*;

    #[test]
    fn add_children() {
        let tpl = "
            let list = [2, 2, 30, 40]
            node
                for x in list
                    for y in [1]
                        node x
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
