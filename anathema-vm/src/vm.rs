use anathema_compiler::{Constants, Instruction};
use anathema_widget_core::expressions::Expression;

use crate::ViewTemplates;
use crate::error::Result;
use crate::scope::Scope;

pub struct VirtualMachine {
    instructions: Vec<Instruction>,
    consts: Constants,
}

impl VirtualMachine {
    pub fn new(instructions: Vec<Instruction>, consts: Constants) -> Self {
        Self {
            instructions,
            consts,
        }
    }

    pub fn exec(self, views: &mut ViewTemplates) -> Result<Vec<Expression>> {
        let mut root_scope = Scope::new(self.instructions, &self.consts);
        root_scope.exec(views)
    }
}

#[cfg(test)]
mod test {
    use anathema_compiler::compile;
    use anathema_widget_core::generator::SingleNode;

    use super::*;

    #[test]
    fn nodes() {
        let (instructions, consts) = compile("vstack").unwrap();
        let vm = VirtualMachine::new(instructions, consts);
        let vstack = vm.exec().unwrap().remove(0);

        assert!(matches!(vstack, Expression::Node(SingleNode { .. })));
    }

    #[test]
    fn for_loop() {
        let src = "
        for x in {{ y }}
            border
        ";
        let (instructions, consts) = compile(src).unwrap();
        let vm = VirtualMachine::new(instructions, consts);
        let for_loop = vm.exec().unwrap().remove(0);

        assert!(matches!(for_loop, Expression::Loop { .. }));
    }
}
