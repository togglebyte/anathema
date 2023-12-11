use anathema_render::Size;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Padding {
    /// Top padding
    pub top: u16,
    /// Right padding
    pub right: u16,
    /// Bottom padding
    pub bottom: u16,
    /// Left padding
    pub left: u16,
}

impl Padding {
    /// Zero padding
    pub const ZERO: Padding = Self::new(0);

    /// Create a new instance padding
    pub const fn new(padding: u16) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    /// Return the current padding and set the padding to zero
    pub fn take(&mut self) -> Self {
        let mut padding = Padding::ZERO;
        std::mem::swap(&mut padding, self);
        padding
    }

    pub fn size(&self) -> Size {
        Size {
            width: (self.left + self.right) as usize,
            height: (self.top + self.bottom) as usize,
        }
    }
}

impl FromIterator<u16> for Padding {
    fn from_iter<T: IntoIterator<Item = u16>>(iter: T) -> Self {
        let mut iter = iter.into_iter();

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
            ValueRef::Owned(Owned::Num(n)) => Some(Self::new(n.to_u16())),
            ValueRef::Expressions(values) => {
                let padding = values
                    .iter()
                    .map(|expr| Resolver::new(context, node_id).resolve(expr))
                    .map(|val| match val {
                        ValueRef::Owned(Owned::Num(n)) => n.to_u16(),
                        _ => 0,
                    });

                Some(Padding::from_iter(padding))
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
        value: &mut Value<Self>,
        context: &Context<'_, '_>,
        node_id: Option<&anathema_values::NodeId>,
    ) where
        Self: Sized,
    {
        if let Value::Dyn { inner, expr } = value {
            let mut resolver = Resolver::new(context, node_id);
            let value = resolver.resolve(expr);
            *inner = match value {
                ValueRef::Owned(Owned::Num(n)) => Some(Self::new(n.to_u16())),
                ValueRef::Expressions(values) => {
                    let padding = values
                        .iter()
                        .map(|expr| Resolver::new(context, node_id).resolve(expr))
                        .map(|val| match val {
                            ValueRef::Owned(Owned::Num(n)) => n.to_u16(),
                            _ => 0,
                        });

                    Some(Padding::from_iter(padding))
                }
                _ => None,
            };
        }
    }
}

#[cfg(feature = "testing")]
#[cfg(test)]
mod test {
    use anathema_values::testing::{unum, TestState};

    use super::*;

    #[test]
    fn padding_from_iter() {
        let actual = Padding::from_iter(vec![1]);
        let expected = Padding {
            top: 1,
            right: 1,
            bottom: 1,
            left: 1,
        };
        assert_eq!(expected, actual);

        let actual = Padding::from_iter(vec![1, 2]);
        let expected = Padding {
            top: 1,
            right: 2,
            bottom: 1,
            left: 2,
        };
        assert_eq!(expected, actual);

        let actual = Padding::from_iter(vec![1, 2, 3]);
        let expected = Padding {
            top: 1,
            right: 2,
            bottom: 3,
            left: 2,
        };
        assert_eq!(expected, actual);

        let actual = Padding::from_iter(vec![1, 2, 3, 4]);
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
