use anathema_state::{Change, States};
use anathema_store::tree::{NodePath, PathFinder};

use super::element::Element;
use super::eval::EvalContext;
use super::loops::LOOP_INDEX;
use crate::components::ComponentRegistry;
use crate::values::ValueId;
use crate::widget::FloatingWidgets;
use crate::{AttributeStorage, Factory, Scope, ValueStack, WidgetKind, WidgetTree};

struct UpdateTree<'a, 'b, 'bp> {
    value_id: ValueId,
    change: &'a Change,
    factory: &'a Factory,
    scope: &'b mut Scope<'bp>,
    value_store: &'b mut ValueStack<'bp>,
    states: &'b mut States,
    components: &'b mut ComponentRegistry,
    attribute_storage: &'b mut AttributeStorage<'bp>,
    floating_widgets: &'b mut FloatingWidgets,
}

impl<'a, 'b, 'bp> PathFinder<WidgetKind<'bp>> for UpdateTree<'a, 'b, 'bp> {
    fn apply(&mut self, node: &mut WidgetKind<'bp>, path: &NodePath, tree: &mut WidgetTree<'bp>) {
        scope_value(node, self.scope, &[]);
        let mut ctx = EvalContext {
            factory: self.factory,
            scope: self.scope,
            value_store: self.value_store,
            states: self.states,
            components: self.components,
            attribute_storage: self.attribute_storage,
            floating_widgets: self.floating_widgets,
        };
        update_widget(node, self.change, self.value_id, &mut ctx, path, tree);
    }

    fn parent(&mut self, parent: &WidgetKind<'bp>, children: &[u16]) {
        scope_value(parent, self.scope, children);
    }
}

/// Scan the widget tree using the node path.
/// Build up the scope from the parent nodes.
pub fn update_tree<'bp>(
    factory: &Factory,
    scope: &mut Scope<'bp>,
    states: &mut States,
    components: &mut ComponentRegistry,
    change: &Change,
    value_store: &mut ValueStack<'bp>,
    value_id: ValueId,
    path: &NodePath,
    tree: &mut WidgetTree<'bp>,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
) {
    let update = UpdateTree {
        value_id,
        change,
        factory,
        scope,
        value_store,
        states,
        components,
        attribute_storage,
        floating_widgets,
    };
    tree.apply_path_finder(path, update);
}

fn update_widget<'bp>(
    widget: &mut WidgetKind<'bp>,
    change: &Change,
    value_id: ValueId,
    ctx: &mut EvalContext<'_, '_, 'bp>,
    path: &NodePath,
    tree: &mut WidgetTree<'bp>,
) {
    match widget {
        WidgetKind::Element(..) => {
            // TODO an update should trigger a possible reflow of the layout
        }
        WidgetKind::For(for_loop) => for_loop.update(ctx, change, value_id, path, tree),
        WidgetKind::Iteration(_) => todo!(),
        WidgetKind::ControlFlow(widget) => widget.update(path, tree),
        WidgetKind::If(_if_widget) => todo!(),
        WidgetKind::Else(_else_widget) => todo!(),
        WidgetKind::Component(_) => {
            // let o = anathema_debug::Debug::new(String::new())
            //     .debug("owned store", DebugOwnedStore)
            //     .debug("shared store", DebugSharedStore)
            //     .finish();
            // eprintln!("{o}");
            panic!("{change:?}");
        }
    }
}

pub(super) fn scope_value<'bp>(widget: &WidgetKind<'bp>, scope: &mut Scope<'bp>, children: &[u16]) {
    match widget {
        WidgetKind::For(for_loop) => match children {
            [next, ..] => {
                let index = *next as usize;
                for_loop.collection.scope(scope, for_loop.binding, index);
            }
            [] => {}
        },
        WidgetKind::Iteration(iter) => {
            scope.scope_pending(LOOP_INDEX, iter.loop_index.to_pending());
        }
        WidgetKind::Component(component) => {
            if let Some(state) = &component.external_state {
                for ((k, _), v) in state.iter() {
                    let v = v.downgrade();
                    scope.scope_downgrade(k, v);
                }
            }
        }
        WidgetKind::ControlFlow(_) | WidgetKind::Element(Element { .. }) | WidgetKind::If(_) | WidgetKind::Else(_) => {}
    }
}
