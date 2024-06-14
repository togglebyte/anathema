#[allow(unused_extern_crates)]
extern crate anathema_state as anathema;
use anathema_widgets::Factory;

mod alignment;
mod border;
mod expand;
mod layout;
mod padding;
mod position;
mod spacer;
mod stacks;
mod text;
mod viewport;

#[cfg(test)]
mod testing;

pub use alignment::Align;
pub use border::Border;
pub use expand::Expand;
pub use padding::Padding;
pub use position::Position;
pub use stacks::{Column, HStack, Row, VStack};
pub use text::Text;
pub use viewport::Viewport;

pub fn register_default_widgets(factory: &mut Factory) {
    factory.register_default::<alignment::Align>("align");
    factory.register_default::<expand::Expand>("expand");
    factory.register_default::<padding::Padding>("padding");
    factory.register_default::<position::Position>("position");
    factory.register_default::<stacks::Column>("column");
    factory.register_default::<spacer::Spacer>("spacer");
    factory.register_default::<stacks::HStack>("hstack");
    factory.register_default::<stacks::Row>("row");
    factory.register_default::<stacks::VStack>("vstack");
    factory.register_default::<stacks::ZStack>("zstack");
    factory.register_default::<text::Span>("span");
    factory.register_default::<text::Text>("text");
    factory.register_default::<viewport::Viewport>("viewport");

    factory.register_widget("border", border::make);
}
