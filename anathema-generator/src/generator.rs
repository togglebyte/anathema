use anathema_values::{BucketRef, ScopeId};

use crate::{Expression, NodeKind};

enum State {
    Block,
    Loop { scope_id: ScopeId },
}

pub enum Op<W> {
    Add(W),
    StartCollection(ScopeId),
    EndCollection,
}

pub struct Generator<'a, V, T, F, W>
where
    F: Fn(&T) -> Option<W>,
{
    bucket: &'a BucketRef<'a, V>,
    // expressions: Vec<Expression<'a, T>>,
    source: &'a [T],
    state: State,
    factory: F,
    source_index: usize,
}

impl<'a, V, T, F, W> Generator<'a, V, T, F, W>
where
    F: Fn(&T) -> Option<W>,
{
    // pub fn new(bucket: BucketRef<'a, V>, expressions: Vec<Expression<'a, T>>, factory: F) -> Self {
    pub fn new(bucket: &'a BucketRef<'a, V>, source: &'a [T], factory: F) -> Self {
        Self {
            bucket,
            source,
            state: State::Block,
            factory,
            source_index: 0,
        }
    }

    pub fn next(&mut self) -> Option<Op<W>> {
        loop {
            // let expr = self.source[self.source_index].to_expression(&self.bucket);

            // match expr {
            //     Expression::Single(t) => break (self.factory)(t).map(|w| Node::Single(w)),
            //     Expression::Loop { collection, body } => {
            //         let scope_id = self.bucket.new_scope();
            //         self.state = State::Loop { scope_id };
            //     }
            //     _ => {}
            // }
        }
    }
}

// let x = 10;
// let y = [1, 2]
//
// for x in y {
//
// }
