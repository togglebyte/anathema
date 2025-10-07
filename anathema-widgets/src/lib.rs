use anathema_core::elements::RegisteredElements;
pub use border::{border, Border};

mod border;

pub fn register_default_widgets(factory: &mut RegisteredElements) {
    factory.register_default::<Border>("border");
}
