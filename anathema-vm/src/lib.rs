mod error;
mod scope;
mod vm;

use anathema_widget_core::template::Template;
pub use vm::VirtualMachine;

use self::error::Result;

pub fn templates(src: &str) -> Result<Vec<Template>> {
    let (instructions, constants) = anathema_compiler::compile(src)?;
    let vm = VirtualMachine::new(instructions, constants);
    vm.exec()
}
