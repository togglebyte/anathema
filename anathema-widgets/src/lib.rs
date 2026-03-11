use anathema_geometry::Size;
use anathema_runtime::widgets::RegisteredWidgets;
use anathema_runtime::{Constraints, WidgetAttributes};
use border::border;

pub use crate::border::Border;
pub use crate::container::Container;
pub use crate::padding::Padding;
pub use crate::text::Text;

mod border;
mod container;
mod padding;
mod text;
mod textlayout;

fn update_constraints(attributes: WidgetAttributes<'_, '_>, constraints: &mut Constraints) {
    if let Some(min_width) = attributes.get_as::<u32>("min_width") {
        constraints.min.width = constraints.min.width.max(min_width);
    }

    if let Some(min_height) = attributes.get_as::<u32>("min_heigth") {
        constraints.min.height = constraints.min.height.max(min_height);
    }

    if let Some(width) = attributes
        .get_as::<u32>("width")
        .or(attributes.get_as::<u32>("max_width"))
    {
        constraints.try_fit_width(width);
    }

    if let Some(height) = attributes
        .get_as::<u32>("height")
        .or(attributes.get_as::<u32>("max_height"))
    {
        constraints.try_fit_height(height);
    }
}

fn fix_size(mut size: Size, attributes: WidgetAttributes<'_, '_>, constraints: Constraints) -> Size {
    if let Some(width) = attributes.get_as::<u32>("width") {
        size.width = width;
    } else if let Some(width) = attributes.get_as::<u32>("min_width")
        && size.width < width
    {
        size.width = width.min(constraints.max.width);
    }

    if let Some(height) = attributes.get_as::<u32>("height") {
        size.height = height;
    } else if let Some(height) = attributes.get_as::<u32>("min_height")
        && size.height < height
    {
        size.height = height.min(constraints.max.height);
    }

    size
}

pub fn register_default_widgets(factory: &mut RegisteredWidgets) {
    factory.register("border", |attrs| Box::new(Border::new(attrs)));
    factory.register("container", |attrs| Box::new(Container));
    factory.register("padding", |attrs| Box::new(Padding));
    factory.register("text", |attrs| Box::new(Text::new()));
}
