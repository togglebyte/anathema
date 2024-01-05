// -----------------------------------------------------------------------------
//   - Example: input -
//   * Add items to a collection based on user input
// -----------------------------------------------------------------------------
use std::fs::read_to_string;

use anathema::core::{Event, KeyCode, Nodes, View};
use anathema::runtime::Runtime;
use anathema::values::{List, State, StateValue};
use anathema::vm::Templates;

#[derive(Debug, State)]
struct RootState {
    input: StateValue<String>,
    output: List<String>,
}

struct RootView {
    state: RootState,
}

impl View for RootView {
    fn on_event(&mut self, event: Event, _nodes: &mut Nodes<'_>) {
        if let Event::KeyPress(code, ..) = event {
            match code {
                KeyCode::Char(c) => self.state.input.push(c),
                KeyCode::Backspace => drop(self.state.input.pop()),
                KeyCode::Enter => {
                    let input = self.state.input.drain(..).collect();
                    self.state.output.push_back(input);
                }
                _ => {}
            }
        }
    }

    fn state(&self) -> &dyn State {
        &self.state
    }
}

fn main() {
    // Step one: setup a root view and state
    let root_view = RootView {
        state: RootState {
            input: String::new().into(),
            output: List::new(vec![]),
        },
    };

    // Step two: load templates
    let tpl = read_to_string("examples/templates/input.tiny").unwrap();
    let mut templates = Templates::new(tpl, root_view);
    let templates = templates.compile().unwrap();

    // Step three: setup runtime
    let mut runtime = Runtime::new(&templates).unwrap();
    runtime.enable_tabindex = false;

    // Disable the alt screen if the application panics
    // and you want to see the panic message.
    // runtime.enable_alt_screen = false;

    // Step four: start the runtime
    runtime.run().unwrap();
}
