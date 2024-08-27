use std::fmt::Write;
use std::ops::ControlFlow;

use super::ValueId;

pub trait NodeVisitor<T> {
    /// Return control flow.
    /// * `ControlFlow::Continue(())` continue
    /// * `ControlFlow::Break(false)` stop iterating over the children of the current node
    /// * `ControlFlow::Break(true)` stop iterating
    fn visit(&mut self, value: &mut T, path: &[u16], value_id: ValueId) -> ControlFlow<bool>;

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
    fn visit(&mut self, value: &mut T, path: &[u16], _: ValueId) -> ControlFlow<bool> {
        let _ = writeln!(&mut self.output, "{}{path:?}: {value:?}", " ".repeat(self.level * 4),);
        ControlFlow::Continue(())
    }

    fn push(&mut self) {
        self.level += 1;
    }

    fn pop(&mut self) {
        self.level -= 1;
    }
}
