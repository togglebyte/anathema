use std::marker::PhantomData;

use anathema_geometry::Size;
use anathema_store::tree::TreeForEach;
use anathema_strings::HStrings;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Expression, Globals};
use anathema_value_resolver::AttributeStorage;

use crate::layout::{Constraints, EvalCtx, LayoutCtx, LayoutFilter, PositionCtx};
use crate::{Factory, LayoutChildren, PositionChildren, Widget, WidgetId, WidgetKind};
use crate::error::Result;

#[derive(Debug, Default)]
struct TestWidget;

impl Widget for TestWidget {
    fn layout<'bp>(
        &mut self,
        _children: LayoutChildren<'_, 'bp>,
        _: Constraints,
        _: WidgetId,
        _: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        todo!()
    }

    fn position<'bp>(
        &mut self,
        _children: PositionChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _ctx: PositionCtx,
    ) {
        todo!()
    }
}

pub(crate) fn setup_test_factory() -> Factory {
    let mut fac = Factory::new();
    fac.register_default::<TestWidget>("test");
    fac
}
