#[allow(unused_extern_crates)]
extern crate anathema_state as anathema;
use anathema_widgets::Factory;

mod alignment;
mod border;
mod canvas;
mod container;
mod expand;
mod layout;
mod overflow;
mod padding;
mod position;
mod spacer;
mod stacks;
mod text;

#[cfg(test)]
mod testing;

pub(crate) const WIDTH: &str = "width";
pub(crate) const HEIGHT: &str = "height";
pub(crate) const MIN_WIDTH: &str = "min_width";
pub(crate) const MIN_HEIGHT: &str = "min_height";
pub(crate) const MAX_WIDTH: &str = "max_width";
pub(crate) const MAX_HEIGHT: &str = "max_height";
pub(crate) const TOP: &str = "top";
pub(crate) const RIGHT: &str = "right";
pub(crate) const BOTTOM: &str = "bottom";
pub(crate) const LEFT: &str = "left";

pub use alignment::Align;
pub use border::Border;
pub use canvas::Canvas;
pub use expand::Expand;
pub use overflow::Overflow;
pub use padding::Padding;
pub use position::Position;
pub use stacks::{Column, HStack, Row, VStack};
pub use text::Text;

pub fn register_default_widgets(factory: &mut Factory) {
    factory.register_default::<alignment::Align>("align");
    factory.register_default::<expand::Expand>("expand");
    factory.register_default::<canvas::Canvas>("canvas");
    factory.register_default::<container::Container>("container");
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
    factory.register_default::<overflow::Overflow>("overflow");
    factory.register_widget("border", border::make);
}
