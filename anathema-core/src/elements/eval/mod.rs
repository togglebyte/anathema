use self::scope::Scope;
use crate::attributes::Attributes;
use crate::elements::RegisteredElements;
use crate::nodes::NodeId;
use crate::templates::{Blueprint, ExpressionId, For, Single};
use crate::ui::Document;

mod scope;

enum Kind<'bp> {
    For {
        body: &'bp [Blueprint],
        binding: &'bp str,
        data: ExpressionId,
    },
    Element(NodeId),
}

struct Tree<'bp> {
    kind: Kind<'bp>,
}

trait Evaluator {
    type Input<'bp>;

    fn eval<'a, 'bp>(
        &mut self,
        input: Self::Input<'bp>,
        doc: &mut Document<'a>,
        tree: &mut Tree<'bp>,
        factory: &RegisteredElements,
        scope: &mut Scope,
        parent: Option<NodeId>,
    ) -> Result<(), ()>;
}

pub fn eval<'a, 'bp>(
    blueprint: &'bp Blueprint,
    doc: &mut Document<'a>,
    tree: &mut Tree<'bp>,
    factory: &RegisteredElements,
    scope: &mut Scope,
    parent: Option<NodeId>,
) {
    match blueprint {
        Blueprint::Single(single) => todo!(),
        Blueprint::For(_) => todo!(),
        Blueprint::With(with) => todo!(),
        Blueprint::ControlFlow(control_flow) => todo!(),
        Blueprint::Component(component) => todo!(),
        Blueprint::Slot(blueprints) => todo!(),
    }
}

struct SingleEval;

impl Evaluator for SingleEval {
    type Input<'bp> = &'bp Single;

    fn eval<'a, 'bp>(
        &mut self,
        single: Self::Input<'bp>,
        doc: &mut Document<'a>,
        tree: &mut Tree<'bp>,
        factory: &RegisteredElements,
        scope: &mut Scope,
        parent: Option<NodeId>,
    ) -> Result<(), ()> {
        for (key, expr) in single.attributes.iter() {}

        let attributes = Attributes::empty();

        let element = match factory.make(&single.ident, &attributes) {
            Ok(el) => el,
            Err(e) => panic!(),//return Err(ctx.error(e)),
        };

        // * Refs to Tree and Doc should be placed in a single type that can only perform inserts at this point.
        // * Insert into both the tree and the doc
        // * Nodes removed from `doc` should be stored as a removed node
        // and these should be drained and then the nodes should be removed from the tree
        // * The tree needs a better name
        
        // doc.

        for child in &single.children {
            eval(child, doc, tree, factory, scope, parent);
        }

        todo!()
    }
}

struct ForLoop;

impl Evaluator for ForLoop {
    type Input<'bp> = &'bp For;

    fn eval<'a, 'bp>(
        &mut self,
        forloop: Self::Input<'bp>,
        doc: &mut Document<'a>,
        tree: &mut Tree<'bp>,
        factory: &RegisteredElements,
        scope: &mut Scope,
        parent: Option<NodeId>,
    ) -> Result<(), ()> {
        // Resolve collection
        // resolve(forloop.data);
        let collection = [1];

        for val in collection {
            // scope.scope(forloop.binding, val);
            for child in &forloop.body {
                eval(child, doc, tree, factory, scope, parent);
            }
        }

        todo!()
    }
}
