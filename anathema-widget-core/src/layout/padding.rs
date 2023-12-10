use anathema_values::{Context, DynValue, Owned, Resolver, Value, ValueRef};

/// Represents the padding of a widget.
/// Padding is not applicable to `text:` widgets.
/// ```ignore
/// # use anathema_widgets::{Text, Border, BorderStyle, Sides, NodeId, Widget, Padding};
/// let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, 8, 5)
///     .into_container(NodeId::anon());
///
/// // Set the padding to 2 on all sides
/// border.padding = Padding::new(2);
///
/// let text = Text::with_text("hi")
///     .into_container(NodeId::anon());
/// border.add_child(text);
/// ```
/// would output
/// ```text
/// ┌──────┐
/// │      │
/// │  hi  │
/// │      │
/// └──────┘
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Padding {
    /// Top padding
    pub top: usize,
    /// Right padding
    pub right: usize,
    /// Bottom padding
    pub bottom: usize,
    /// Left padding
    pub left: usize,
}

impl Padding {
    /// Zero padding
    pub const ZERO: Padding = Self::new(0);

    /// Create a new instance padding
    pub const fn new(padding: usize) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    pub fn from_iter(mut iter: impl Iterator<Item = usize>) -> Self {
        let Some(n) = iter.next() else {
            return Self::ZERO;
        };
        let mut padding = Self::new(n);

        let Some(right) = iter.next() else {
            return padding;
        };
        padding.right = right;

        let Some(bottom) = iter.next() else {
            padding.bottom = padding.top;
            padding.left = padding.right;
            return padding;
        };

        padding.bottom = bottom;

        let Some(left) = iter.next() else {
            padding.left = padding.right;
            return padding;
        };

        padding.left = left;

        padding
    }

    /// Return the current padding and set the padding to zero
    pub fn take(&mut self) -> Self {
        let mut padding = Padding::ZERO;
        std::mem::swap(&mut padding, self);
        padding
    }
}

impl DynValue for Padding {
    fn init_value(
        context: &Context<'_, '_>,
        node_id: Option<&anathema_values::NodeId>,
        expr: &anathema_values::ValueExpr,
    ) -> Value<Self>
    where
        Self: Sized,
    {
        let mut resolver = Resolver::new(context, node_id);
        let value = resolver.resolve(expr);

        let inner = match value {
            ValueRef::Owned(Owned::Num(n)) => Some(Self::new(n.to_usize())),
            ValueRef::Expressions(values) => {
                values
                    .iter()
                    .map(|expr| Resolver::new(context, node_id).resolve(expr))
                    .map(|val| match val {
                        ValueRef::Owned(Owned::Num(n)) => n.to_usize(),
                        _ => 0,
                    })
                    .collect::<Vec<_>>();
                panic!("come back and deal with padding please")
            }
            _ => None,
        };

        match resolver.is_deferred() {
            true => Value::Dyn {
                inner,
                expr: expr.clone(),
            },
            false => match inner {
                Some(val) => Value::Static(val),
                None => Value::Empty,
            },
        }
    }

    fn resolve(
        _value: &mut Value<Self>,
        _context: &Context<'_, '_>,
        _node_id: Option<&anathema_values::NodeId>,
    ) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::{unum, TestState};

    use super::*;

    #[test]
    fn padding_from_iter() {
        let actual = Padding::from_iter(vec![1].into_iter());
        let expected = Padding {
            top: 1,
            right: 1,
            bottom: 1,
            left: 1,
        };
        assert_eq!(expected, actual);

        let actual = Padding::from_iter(vec![1, 2].into_iter());
        let expected = Padding {
            top: 1,
            right: 2,
            bottom: 1,
            left: 2,
        };
        assert_eq!(expected, actual);

        let actual = Padding::from_iter(vec![1, 2, 3].into_iter());
        let expected = Padding {
            top: 1,
            right: 2,
            bottom: 3,
            left: 2,
        };
        assert_eq!(expected, actual);

        let actual = Padding::from_iter(vec![1, 2, 3, 4].into_iter());
        let expected = Padding {
            top: 1,
            right: 2,
            bottom: 3,
            left: 4,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn resolve_padding() {
        let state = TestState::new();
        let ctx = Context::root(&state);
        let _resolver = Resolver::new(&ctx, None);

        let e = unum(2);
        let actual = Padding::init_value(&ctx, None, &e);

        let expected = Padding::new(2);
        assert_eq!(&expected, actual.value_ref().unwrap());
    }
}
