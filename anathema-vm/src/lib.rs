mod error;
mod scope;
mod vm;

use anathema_generator::Expression;
use anathema_values::StoreMut;
use anathema_widget_core::{Value, WidgetContainer};
pub use vm::VirtualMachine;

use self::error::Result;

pub type Expressions = Vec<Expression<WidgetContainer>>;

pub fn templates(src: &str, mut bucket: StoreMut<'_, Value>) -> Result<Expressions> {
    let (instructions, constants) = anathema_compiler::compile(src)?;
    for path in constants.paths().cloned() {
        bucket.insert_path(path);
    }
    let vm = VirtualMachine::new(instructions, constants);
    vm.exec(&mut bucket)
}
