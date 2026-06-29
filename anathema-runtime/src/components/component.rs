use crate::State;

/// User defined components
///
/// ```
/// use anathema_core::runtime::Component;
/// use anathema_state::Value;
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     type State = Value<String>;
///     type Message = u32;
/// }
/// ```
pub trait Component: 'static {
    /// The state associated with the component
    type State: State;
    /// The type of message that can be sent to the component
    type Message;
}

pub(crate) trait AnyComponent: 'static {
}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<component>")
    }
}

impl<T: Component> AnyComponent for T {
}
