pub trait Component: 'static {
    type State;
    type Message;
}

pub trait AnyComponent: 'static {
}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<component>")
    }
}

impl<T: Component> AnyComponent for T {
}
