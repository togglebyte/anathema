use anathema_widget_core::expressions::Expression;
use anathema_widget_core::testing::{test_widget as core_test_widget, FakeTerm};
use anathema_widget_core::Widget;

pub fn test_widget(expr: Expression, expected: FakeTerm) {
    let _ = crate::register_default_widgets();
    core_test_widget(expr, expected);
}
