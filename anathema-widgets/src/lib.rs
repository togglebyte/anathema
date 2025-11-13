use anathema_runtime::widgets::RegisteredWidgets;
pub use border::{border, Border};

mod border;

pub fn register_default_widgets(factory: &mut RegisteredWidgets) {
    // factory.register("border", |attrs| Border::new(attrs));
}
