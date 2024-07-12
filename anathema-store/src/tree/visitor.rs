use std::fmt::Write;
use std::ops::ControlFlow;

use super::{NodePath, ValueId};

pub trait NodeVisitor<T> {
    fn visit(&mut self, value: &mut T, path: &NodePath, value_id: ValueId) -> ControlFlow<()>;

    fn push(&mut self) {}

    fn pop(&mut self) {}
}

/// Debug print a tree
pub struct DebugPrintTree {
    output: String,
    level: usize,
}

impl DebugPrintTree {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            level: 0,
        }
    }

    pub fn finish(self) -> String {
        self.output
    }
}

impl<T> NodeVisitor<T> for DebugPrintTree
where
    T: std::fmt::Debug,
{
    fn visit(&mut self, value: &mut T, path: &NodePath, _: ValueId) -> ControlFlow<()> {
        let _ = writeln!(
            &mut self.output,
            "{}{path:?}: {value:?}",
            " ".repeat(self.level * 4),
            path = path.as_slice()
        );
        ControlFlow::Continue(())
    }

    fn push(&mut self) {
        self.level += 1;
    }

    fn pop(&mut self) {
        self.level -= 1;
    }
}
