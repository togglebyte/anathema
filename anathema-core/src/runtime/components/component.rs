pub trait Component {
    type State;
    type Message;
}

pub trait AnyComponent {
}

impl<T: Component> AnyComponent for T {
}
