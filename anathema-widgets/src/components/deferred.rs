use std::any::Any;
use std::borrow::Cow;

use anathema_value_resolver::{Attributes, ValueKind};

use crate::{WidgetId, nodes::component::Component};

pub struct DeferredComponents {
    queue: Vec<Command>,
}

impl DeferredComponents {
    pub fn new() -> Self {
        Self { queue: vec![] }
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Command> + '_ {
        self.queue.drain(..).rev()
    }

    pub fn by_name(&mut self, name: impl Into<Cow<'static, str>>) -> QueryBuilder<'_> {
        QueryBuilder::new(&mut self.queue, Filter::Name(name.into()))
    }

    pub fn by_attribute(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<ValueKind<'static>>,
    ) -> QueryBuilder<'_> {
        QueryBuilder::new(
            &mut self.queue,
            Filter::Attribute {
                key: key.into(),
                value: value.into(),
            },
        )
    }

    pub fn nth(&mut self, count: usize) -> QueryBuilder<'_> {
        QueryBuilder::new(&mut self.queue, Filter::Nth(count))
    }

    pub fn by_id(&mut self, id: WidgetId) -> QueryBuilder<'_> {
        QueryBuilder::new(&mut self.queue, Filter::Id(id))
    }
}

pub struct QueryBuilder<'a> {
    queue: &'a mut Vec<Command>,
    filter: Filter,
}

impl<'a> QueryBuilder<'a> {
    fn new(queue: &'a mut Vec<Command>, filter: Filter) -> Self {
        Self { queue, filter }
    }

    pub fn by_name(self, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            queue: self.queue,
            filter: self.filter.chain(Filter::Name(name.into())),
        }
    }

    pub fn by_attribute(self, key: impl Into<Cow<'static, str>>, value: impl Into<ValueKind<'static>>) -> Self {
        Self {
            queue: self.queue,
            filter: self.filter.chain(Filter::Attribute {
                key: key.into(),
                value: value.into(),
            }),
        }
    }

    pub fn nth(self, count: usize) -> Self {
        Self {
            queue: self.queue,
            filter: self.filter.chain(Filter::Nth(count)),
        }
    }

    pub fn by_id(self, id: WidgetId) -> Self {
        Self {
            queue: self.queue,
            filter: self.filter.chain(Filter::Id(id)),
        }
    }

    pub fn send(self, message: impl Any + Send + Sync) {
        let command = Command {
            kind: CommandKind::SendMessage(Box::new(message)),
            filter: self.filter,
        };
        self.queue.push(command);
    }

    pub fn focus(self) {
        let command = Command {
            kind: CommandKind::Focus,
            filter: self.filter,
        };
        self.queue.push(command);
    }
}

enum Filter {
    Name(Cow<'static, str>),
    Attribute {
        key: Cow<'static, str>,
        value: ValueKind<'static>,
    },
    Nth(usize),
    Chain(Box<Self>, Box<Self>),
    Id(WidgetId),
}

impl Filter {
    fn chain(self, other: Self) -> Self {
        Self::Chain(Box::new(self), Box::new(other))
    }

    // The filter only works with primitives, this excludes maps, lists and composite
    // values
    fn filter(&mut self, component: &Component<'_>, attributes: &Attributes<'_>) -> bool {
        match self {
            Filter::Name(cow) => component.name == cow,
            Filter::Attribute { key, value: rhs } => match attributes.get(key) {
                Some(lhs) => match (lhs, rhs) {
                    (ValueKind::Int(lhs), ValueKind::Int(rhs)) => lhs == rhs,
                    (ValueKind::Float(lhs), ValueKind::Float(rhs)) => lhs == rhs,
                    (ValueKind::Bool(lhs), ValueKind::Bool(rhs)) => lhs == rhs,
                    (ValueKind::Char(lhs), ValueKind::Char(rhs)) => lhs == rhs,
                    (ValueKind::Hex(lhs), ValueKind::Hex(rhs)) => lhs == rhs,
                    (ValueKind::Str(lhs), ValueKind::Str(rhs)) => lhs == rhs,
                    (ValueKind::Null, ValueKind::Null) => true,
                    _ => false,
                },
                None => false,
            },
            Filter::Nth(0) => true,
            Filter::Nth(nth) => {
                *nth -= 1;
                false
            }
            Filter::Chain(first, second) => match first.filter(component, attributes) {
                true => second.filter(component, attributes),
                false => false,
            },
            Filter::Id(widget_id) => component.widget_id == *widget_id,
        }
    }
}

pub struct Command {
    filter: Filter,
    pub kind: CommandKind,
}

impl Command {
    pub fn filter_component(&mut self, component: &Component<'_>, attributes: &Attributes<'_>) -> bool {
        self.filter.filter(component, attributes)
    }
}

pub enum CommandKind {
    SendMessage(Box<dyn Any + Send + Sync>),
    Focus,
}
