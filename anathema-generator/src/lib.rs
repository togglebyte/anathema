pub use crate::expression::Expression;
pub use crate::generator::Generator;
pub use crate::nodes::{Node, Nodes};

mod expression;
mod generator;
mod nodes;
mod testing;


struct SometimesIJustWantToQuit {
    inner: Vec<String>,
}

impl SometimesIJustWantToQuit {
    fn next(&mut self) -> Option<&mut String> {
        if self.inner.is_empty() {
            self.inner.push(String::new());
        }

        let index = self.inner.len() - 1;

        Some(&mut self.inner[index])
    }
}
