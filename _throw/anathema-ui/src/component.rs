// -----------------------------------------------------------------------------
//   - Component -
// -----------------------------------------------------------------------------

use crate::UI;

pub trait Component {
    type State;
    type Message;

    fn on_tick(&mut self, ui: &mut UI<'_>);
}

// -----------------------------------------------------------------------------
//   - Any component (erased type) -
// -----------------------------------------------------------------------------

pub trait AnyComponent {
    fn on_tick(&mut self, ui: &mut UI<'_>);
}

impl<T: Component> AnyComponent for T {
    fn on_tick(&mut self, ui: &mut UI<'_>) {
        T::on_tick(self, ui)
    }
}

#[test]
fn here() {
    struct C;
    impl Component for C {
        type State = ();
        type Message = ();

        fn on_tick(&mut self, ui: &mut UI<'_>) {
            // Add a new node
            vstack().add_child(text("lol")).add_child(text("hello")).to_element();
            let node = text("lol").to_element();
            let node = component("compname", SomeComponent::new(), SomeState::new()).to_node();
            tree.this().add_child(node);
            let bb = tree.this().bounding_box();
        }
    }
}
