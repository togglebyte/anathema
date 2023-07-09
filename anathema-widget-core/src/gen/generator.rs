use super::scope::Scope;
use super::store::Store;
use crate::error::Result;
use crate::template::Template;
use crate::WidgetContainer;

// -----------------------------------------------------------------------------
//   - Direction -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Forward,
    Backward,
}

// -----------------------------------------------------------------------------
//   - Generator -
// -----------------------------------------------------------------------------
pub struct Generator<'parent> {
    scope: Scope<'parent>,
}

impl<'parent> Generator<'parent> {
    pub fn new(templates: &'parent [Template], values: &mut Store<'parent>) -> Self {
        Self {
            scope: Scope::new(templates, values, Direction::Forward),
        }
    }

    /// Reverse the generator from its current position
    pub fn reverse(&mut self) {
        self.scope.reverse();
    }

    /// Flip the generator to start from the end and change direction.
    pub fn flip(&mut self) {
        self.scope.flip();
    }

    pub fn next(&mut self, values: &mut Store<'parent>) -> Option<Result<WidgetContainer>> {
        self.scope.next(values)
    }
}
