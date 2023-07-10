use anathema_widget_core::template::Template;
use anathema_widget_core::Widget;
use anathema_widget_core::testing::{test_widget as core_test_widget, FakeTerm};

pub fn test_widget(
    widget: impl Widget + 'static + PartialEq,
    children: impl Into<Vec<Template>>,
    expected: FakeTerm,
) {
    let _ = crate::register_default_widgets();
    core_test_widget(widget, children, expected);
}
