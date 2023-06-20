use anathema_compiler::{Constants, Instruction};
use anathema_widgets::template::Template;
use anathema_widgets::NodeId;

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

    pub fn exec(self) -> Result<Vec<Template>> {
        let mut root_scope = Scope::new(self.instructions, &self.consts);
        root_scope.exec(NodeId::empty())
    }
}

#[cfg(test)]
mod test {
    use anathema_compiler::compile;
    use anathema_widgets::template::Template;

    use super::*;

    #[test]
    fn nodes() {
        let (instructions, consts) = compile("vstack").unwrap();
        let vm = VirtualMachine::new(instructions, consts);
        let vstack_gen = vm.exec().unwrap().remove(0);

        let Template::Node { ident, .. } = vstack_gen else { panic!("wrong kind") };

        assert_eq!(ident, "vstack");
    }

    #[test]
    fn for_loop() {
        let src = "
        for x in {{ y }}
            border
        ";
        let (instructions, consts) = compile(src).unwrap();
        let vm = VirtualMachine::new(instructions, consts);
        let mut for_loop = vm.exec().unwrap().remove(0);

        assert!(matches!(
            for_loop,
            Template::Loop { .. }
        ));

        let Template::Loop { binding, .. } = for_loop else { panic!("wrong kind") };

        assert_eq!(binding, "x");
    }
}
