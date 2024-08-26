use anathema_store::tree::{Node, TreeValues};
use anathema_templates::blueprints::Blueprint;

use crate::expressions::EvalValue;
use crate::{Value, WidgetKind};

#[derive(Debug)]
pub struct ControlFlow;

impl ControlFlow {
    pub(crate) fn update(&self, children: &[Node], values: &mut TreeValues<WidgetKind<'_>>) {
        // Once an if / else is set to true, everything else should be set to false.
        let mut was_set = false;

        for node in children {
            let Some((_, widget)) = values.get_mut(node.value()) else { continue };
            match widget {
                WidgetKind::If(widget) => {
                    if widget.is_true() {
                        widget.show = true;
                        was_set = true;
                    } else {
                        widget.show = false;
                    }
                }
                WidgetKind::Else(widget) => {
                    if was_set {
                        widget.show = false;
                    } else if widget.is_true() {
                        widget.show = true;
                        was_set = true;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug)]
pub struct If<'bp> {
    pub cond: Value<'bp, EvalValue<'bp>>,
    pub show: bool,
}

impl If<'_> {
    pub(crate) fn is_true(&self) -> bool {
        self.cond.load_common_val().map(|v| v.load_bool()).unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct Else<'bp> {
    pub cond: Option<Value<'bp, EvalValue<'bp>>>,
    pub body: &'bp [Blueprint],
    pub show: bool,
}

impl Else<'_> {
    pub(crate) fn is_true(&self) -> bool {
        match self.cond.as_ref() {
            Some(cond) => cond.load_common_val().map(|v| v.load_bool()).unwrap_or(false),
            None => true,
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_state::{Map, States};
    use anathema_store::tree::Tree;
    use anathema_templates::Document;

    use crate::components::ComponentRegistry;
    use crate::nodes::stringify::Stringify;
    use crate::scope::Scope;
    use crate::testing::setup_test_factory;
    use crate::{eval_blueprint, AttributeStorage, Components, EvalContext, FloatingWidgets};

    #[test]
    fn if_stmt() {
        let tpl = "
        if a
            test a
            test a
            test a
            test a
        else
            test
            test !a
        ";
        let mut map = Map::empty();
        map.insert("a", true);

        let mut doc = Document::new(tpl);
        let (blueprint, globals) = doc.compile().unwrap();
        let mut widget_tree = Tree::<_>::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();
        let factory = setup_test_factory();
        let mut component_registry = ComponentRegistry::new();
        let mut components = Components::new();
        let mut states = States::new();
        let state_id = states.insert(Box::new(map));
        let mut scope = Scope::new();
        scope.insert_state(state_id);

        let mut ctx = EvalContext::new(
            &globals,
            &factory,
            &mut scope,
            &mut states,
            &mut component_registry,
            &mut attribute_storage,
            &mut floating_widgets,
            &mut components,
        );

        eval_blueprint(&blueprint, &mut ctx, &[], &mut widget_tree).unwrap();

        let mut stringify = Stringify::new(&attribute_storage);
        widget_tree.apply_visitor(&mut stringify);
        let output = stringify.finish();

        let expected = "
<control flow>
    <if cond = true>
        test Bool(true)
        test Bool(true)
        test Bool(true)
        test Bool(true)
    <else>
        test
        test Bool(false)
    ";

        assert_eq!(expected.trim(), output.trim());
    }
}
