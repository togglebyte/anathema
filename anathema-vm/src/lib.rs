mod error;
mod scope;
mod vm;

use anathema_generator::Expression;
use anathema_values::BucketMut;
use anathema_widget_core::template::Template;
use anathema_widget_core::{Attributes, Value, WidgetMeta};
pub use vm::VirtualMachine;

use self::error::Result;

pub fn templates(
    src: &str,
    mut bucket: BucketMut<'_, Value>,
) -> Result<Vec<Expression<WidgetMeta, Value>>> {
    let (instructions, constants) = anathema_compiler::compile(src)?;
    for path in constants.paths().cloned() {
        bucket.insert_path(path);
    }
    let vm = VirtualMachine::new(instructions, constants);
    vm.exec(&mut bucket)
}
