// Associate expressions with elements
use std::hash::Hash;

use anathema_compiler::expressions::ExpressionId;
use anathema_hashmap::{HashMap, HashSet};

use super::values::TemplateValue;
use crate::elements::ElementId;

#[derive(Debug)]
enum Entry<T> {
    Empty,
    One(T),
    Many(HashSet<T>),
}

impl<T> Entry<T>
where
    T: PartialEq + Copy + Eq,
    T: Hash,
{
    fn add(&mut self, value: T) {
        match self {
            Self::Empty => *self = Self::One(value),
            Self::One(existing) => *self = Self::Many(HashSet::from_iter([*existing, value])),
            Self::Many(items) => _ = items.insert(value),
        }
    }

    fn remove(&mut self, value: T) {
        match self {
            Self::One(existing) if *existing == value => *self = Self::Empty,
            Self::Many(values) => _ = values.remove(&value),
            _ => return,
        }
    }

    fn extend_other(&self, source: &mut Vec<T>)
    where
        T: Copy,
    {
        match self {
            Entry::Empty => return,
            Entry::One(val) => source.push(*val),
            Entry::Many(items) => source.extend(items),
        }
    }

    fn for_each(&self, mut f: impl FnMut(T)) {
        match self {
            Self::Empty => (),
            Self::One(val) => f(*val),
            Self::Many(values) => values.iter().copied().for_each(f),
        }
    }
}

// Associate expressions with elements
#[derive(Debug)]
pub(super) struct Associations {
    expressions: HashMap<ExpressionId, Entry<ElementId>>,
    elements: HashMap<ElementId, Entry<ExpressionId>>,
}

impl Associations {
    pub fn new() -> Self {
        Self {
            expressions: HashMap::default(),
            elements: HashMap::default(),
        }
    }

    fn add_expression(&mut self, expression: ExpressionId) {
        self.expressions.insert(expression, Entry::Empty);
    }

    pub(super) fn associate(&mut self, expr: ExpressionId, el: ElementId) -> Option<()> {
        let expression = self.expressions.get_mut(&expr)?;
        expression.add(el);

        let element = self.elements.get_mut(&el)?;
        element.add(expr);

        Some(())
    }

    fn disassociate(&mut self, el: ElementId) -> Option<()> {
        let expressions = self.elements.remove(&el)?;

        expressions.for_each(|expr: ExpressionId| {
            if let Some(expressions) = self.expressions.get_mut(&expr) {
                expressions.remove(el);
            }
        });

        Some(())
    }
}
