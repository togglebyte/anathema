mod error;
mod scope;
mod vm;

use anathema_generator::Expression;
use anathema_widget_core::WidgetContainer;
pub use vm::VirtualMachine;

use self::error::Result;

pub type Expressions = Vec<Expression<WidgetContainer>>;

pub fn templates(src: &str) -> Result<Expressions> {
    let (instructions, constants) = anathema_compiler::compile(src)?;
    let vm = VirtualMachine::new(instructions, constants);
    vm.exec()
}
