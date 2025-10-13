use anathema_core::runtime::widgets::RegisteredWidgets;
pub use border::{border, Border};

mod border;

pub fn register_default_widgets(factory: &mut RegisteredWidgets) {
    factory.register_default::<Border>("border");
}
