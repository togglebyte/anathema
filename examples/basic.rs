use std::fs::read_to_string;

use anathema::runtime::Runtime;
use anathema::vm::Templates;

fn main() {
    // Step one: Load and compile templates
    let template = read_to_string("examples/templates/basic.tiny").unwrap();
    let mut templates = Templates::new(template, ());
    templates.compile().unwrap();

    // Step two: Runtime
    let runtime = Runtime::new(templates.expressions()).unwrap();

    // Step three: start the runtime
    runtime.run().unwrap();
}
