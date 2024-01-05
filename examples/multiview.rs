// -----------------------------------------------------------------------------
//   - Multiview example -
//   This example is a bit more "complex" and has more moving parts.
//   It shows off the use of multiple views.
//
//   For a more terse example see basic.rs
// -----------------------------------------------------------------------------
use std::fs::read_to_string;

use anathema::core::View;
use anathema::runtime::Runtime;
use anathema::values::{List, State, StateValue};
use anathema::vm::Templates;

// -----------------------------------------------------------------------------
//   - Root -
//   Root view and state(s)
//
//   The `Item` is part of the `RootState` as well as the
//   state for the `ItemView`
// -----------------------------------------------------------------------------
#[derive(Debug, State)]
struct Item {
    index: StateValue<usize>,
}

#[derive(Debug, State)]
struct RootState {
    items: List<Item>,
}

struct RootView {
    state: RootState,
}

impl RootView {
    pub fn new() -> Self {
        Self {
            state: RootState {
                items: List::new(vec![
                    Item { index: 0.into() },
                    Item { index: 1.into() },
                    Item { index: 2.into() },
                ]),
            },
        }
    }
}

impl View for RootView {
    fn state(&self) -> &dyn State {
        &self.state
    }
}

// -----------------------------------------------------------------------------
//   - Item -
//   Item view
// -----------------------------------------------------------------------------
#[derive(Debug)]
struct ItemView {}

impl ItemView {
    pub fn new() -> Self {
        Self {}
    }
}

impl View for ItemView {}

fn main() {
    // -----------------------------------------------------------------------------
    //   - Templates -
    // -----------------------------------------------------------------------------
    let root = read_to_string("examples/templates/multiview/root.tiny").unwrap();
    let items = read_to_string("examples/templates/multiview/items.tiny").unwrap();
    let item = read_to_string("examples/templates/multiview/item.tiny").unwrap();
    let mut templates = Templates::new(root, RootView::new());

    // Add a single view at setup time for the items.
    // Since this view contains no internal state nor does it do
    // any event handling, it's possible to pass a unit as the view.
    templates.add_view("itemlist", items, ());

    // A prototype describes how to create a view when it's needed.
    // Use this when one or more views are created as a result of
    // some condition or loop
    templates.add_prototype("item", item, ItemView::new);

    let templates = templates.compile().unwrap();

    // -----------------------------------------------------------------------------
    //   - Runtime -
    // -----------------------------------------------------------------------------
    let runtime = Runtime::new(&templates).unwrap();

    // -----------------------------------------------------------------------------
    //   - Start -
    // -----------------------------------------------------------------------------
    runtime.run().unwrap();
}
