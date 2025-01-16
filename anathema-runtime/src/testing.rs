use anathema_state::State;
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::Globals;
use anathema_widgets::components::events::Event;
use anathema_widgets::{eval_blueprint, AttributeStorage, Stringify, WidgetTree};

pub struct RuntimeSetup {
    globals: Globals,
    blueprint: Blueprint,
}

impl RuntimeSetup {
    pub fn new(template: impl AsRef<str>) -> Self {
        let template = template.as_ref();
        let (blueprint, globals) = panic!();
        Self { blueprint, globals }
    }
}

pub struct TestRuntime<'bp> {
    tree: WidgetTree<'bp>,
    setup: &'bp RuntimeSetup,
    attribute_storage: AttributeStorage<'bp>,
}

impl<'bp> TestRuntime<'bp> {
    fn new(setup: &'bp RuntimeSetup) -> Self {
        let mut tree = WidgetTree::empty();
        let mut view = tree.view_mut();
        let mut ctx = panic!();
        let res = eval_blueprint(&setup.blueprint, &mut ctx, panic!("missing scope"), root_node(), &mut view);
        Self {
            tree,
            setup,
            attribute_storage: AttributeStorage::empty(),
        }
    }

    pub fn expect_tree(&mut self, tree: &str) {
        let mut stringify = Stringify::new(&self.attribute_storage);
        self.tree.apply_visitor(&mut stringify);
        let output = stringify.finish();
        assert_eq!(output.trim(), tree.trim(), "tree does match the output");
    }

    pub fn event(&mut self, event: Event) {}

    pub fn next_frame(&mut self) {}
}
