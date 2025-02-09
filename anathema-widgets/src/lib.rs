use anathema_state::Subscriber;

pub use crate::nodes::{
    eval_blueprint, update_widget, Element, Stringify, WidgetContainer, WidgetGenerator, WidgetKind,
};
pub use crate::paint::{GlyphMap, WidgetRenderer};
pub use crate::widget::{
    AnyWidget, Attributes, ComponentParents, Components, DirtyWidgets, Factory, FloatingWidgets, ForEach,
    LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Style, Widget, WidgetId, WidgetTree,
    WidgetTreeView,
};

pub type ChangeList = anathema_store::regionlist::RegionList<32, WidgetId, Subscriber>;

#[cfg(test)]
mod testing;

pub mod components;
mod container;
pub mod error;
pub mod layout;
mod nodes;
pub mod paint;
pub mod query;
pub mod tree;
mod widget;

#[cfg(feature = "debuggy")]
pub mod macros {
    #[macro_export]
    macro_rules! awful_debug {
        ($($arg:tt)*) => {
            use ::std::io::Write as _;
            let mut file = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/log.lol").unwrap();
            let payload = format!($($arg)*);
            file.write_all(payload.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
            file.flush();
        }
    }

    #[macro_export]
    macro_rules! debug_tree {
        ($tree:expr) => {
            let mut d = $crate::tree::debug::DebugTree::new();
            $tree.apply_visitor(&mut d);
            $crate::awful_debug!("{}", d.output);
        }
    }
}

#[cfg(not(feature = "debuggy"))]
pub mod macros {
    #[macro_export]
    macro_rules! awful_debug {
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! debug_tree {
        ($tree:expr) => {}
    }
}
