use anathema_store::tree::{Node, TreeValues};
use anathema_templates::blueprints::{self, Blueprint};
use anathema_value_resolver::Value;

use super::WidgetContainer;
use crate::widget::WidgetTreeView;
use crate::WidgetKind;

#[derive(Debug)]
pub struct ControlFlow<'bp> {
    pub elses: Vec<Else<'bp>>,
}

impl ControlFlow<'_> {
    pub(crate) fn has_changed(&self, children: &WidgetTreeView<'_, '_>) -> bool {
        let child_count = children.layout_len();
        if child_count != 1 {
            return true;
        }

        let branch_id = self.current_branch_id(children);

        // Check if another branch id before this has become true,
        // if so this has changed.
        if self.elses[..branch_id as usize].iter().any(|e| e.is_true()) {
            return true;
        }

        // If the current branch is false, the value has changed,
        // as it has to have been true at one point to become
        // the current branch.
        !self.elses[branch_id as usize].is_true()
    }

    fn current_branch_id(&self, children: &WidgetTreeView<'_, '_>) -> u16 {
        let node_id = children.layout[0].value();
        let (path, widget) = children
            .values
            .get(node_id)
            .expect("because the node exists, the value exist");

        let WidgetKind::ControlFlowContainer(id) = widget.kind else { unreachable!() };
        id
    }
}

// impl ControlFlow<'_> {
//     pub(crate) fn update(&self, children: &[Node], values: &mut TreeValues<WidgetContainer<'_>>) {
//         // Once an if / else is set to true, everything else should be set to false.
//         let mut was_set = false;

//         for node in children {
//             let Some((_, widget)) = values.get_mut(node.value()) else { continue };
//             match &mut widget.kind {
//                 WidgetKind::If(widget) => {
//                     if widget.is_true() {
//                         widget.show = true;
//                         was_set = true;
//                     } else {
//                         widget.show = false;
//                     }
//                 }
//                 WidgetKind::Else(widget) => {
//                     if was_set {
//                         widget.show = false;
//                     } else if widget.is_true() {
//                         widget.show = true;
//                         was_set = true;
//                     }
//                 }
//                 _ => unreachable!(),
//             }
//         }
//     }
// }

// #[derive(Debug)]
// pub struct If<'bp> {
//     pub cond: Value<'bp, EvalValue<'bp>>,
//     pub body: &'bp [Blueprint],
//     pub show: bool,
// }

// impl If<'_> {
//     pub(crate) fn is_true(&self) -> bool {
//         self.cond.load_common_val().map(|v| v.load_bool()).unwrap_or(false)
//     }
// }

#[derive(Debug)]
pub struct Else<'bp> {
    pub cond: Option<Value<'bp>>,
    pub body: &'bp [Blueprint],
    pub show: bool,
}

impl Else<'_> {
    pub(crate) fn is_true(&self) -> bool {
        match self.cond.as_ref() {
            Some(cond) => cond.as_bool().unwrap_or(false),
            None => true,
        }
    }
}

// #[cfg(test)]
// mod test {
//     use anathema_state::{Map, States};
//     use anathema_store::tree::Tree;
//     use anathema_templates::Document;

//     use crate::components::ComponentRegistry;
//     use crate::nodes::stringify::Stringify;
//     use crate::scope::Scope;
//     use crate::testing::setup_test_factory;
//     use crate::{eval_blueprint, AttributeStorage, Components, DirtyWidgets, EvalContext, FloatingWidgets};

//     #[test]
//     fn if_stmt() {
//         let tpl = "
//         if state.a
//             test state.a
//             test state.a
//             test state.a
//             test state.a
//         else
//             test
//             test !state.a
//         ";
//         let mut map = Map::empty();
//         map.insert("a", true);

//         let mut doc = Document::new(tpl);
//         let (blueprint, globals) = doc.compile().unwrap();
//         let mut widget_tree = Tree::<_>::empty();
//         let mut attribute_storage = AttributeStorage::empty();
//         let mut floating_widgets = FloatingWidgets::empty();
//         let factory = setup_test_factory();
//         let mut component_registry = ComponentRegistry::new();
//         let mut components = Components::new();
//         let mut dirty_widgets = DirtyWidgets::empty();
//         let mut states = States::new();
//         let state_id = states.insert(Box::new(map));
//         let mut scope = Scope::new();
//         scope.insert_state(state_id);

//         let mut ctx = EvalContext::new(
//             &globals,
//             &factory,
//             &mut scope,
//             &mut states,
//             &mut component_registry,
//             &mut attribute_storage,
//             &mut floating_widgets,
//             &mut components,
//             &mut dirty_widgets,
//         );

//         eval_blueprint(&blueprint, &mut ctx, &[], &mut widget_tree).unwrap();

//         let mut stringify = Stringify::new(&attribute_storage);
//         widget_tree.apply_visitor(&mut stringify);
//         let output = stringify.finish();

//         let expected = "
// <control flow>
//     <if cond = true>
//         test Bool(true)
//         test Bool(true)
//         test Bool(true)
//         test Bool(true)
//     <else>
//         test
//         test Bool(false)
//     ";

//         assert_eq!(expected.trim(), output.trim());
//     }
// }
