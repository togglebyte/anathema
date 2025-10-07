use std::fmt::Write;
use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_state::States;
use anathema_store::tree::root_node;
use anathema_templates::Document;
use anathema_value_resolver::{AttributeStorage, FunctionTable, Scope};

use crate::components::ComponentRegistry;
use crate::layout::{LayoutCtx, Viewport};
use crate::{
    Components, Factory, FloatingWidgets, GlyphMap, LayoutForEach, Widget, WidgetTree, WidgetTreeView, eval_blueprint,
};

pub fn with_template<F>(tpl: &'static str, mut f: F)
where
    F: for<'bp> FnMut(WidgetTreeView<'_, 'bp>, &mut AttributeStorage<'bp>),
{
    let mut tree = WidgetTree::empty();
    let doc = Box::leak(Document::new(tpl).into());
    let mut variables = Default::default();
    let blueprint = doc.compile(&mut variables).unwrap();
    let variables = Box::leak(Box::new(variables));
    let blueprint = Box::leak(Box::new(blueprint));
    let function_table = Box::leak(Box::new(FunctionTable::new()));

    let mut factory = Factory::new();
    factory.register_default::<Many>("many");
    factory.register_default::<Float>("float");
    factory.register_default::<Text>("text");

    let mut states = States::new();
    let mut attribute_storage = AttributeStorage::empty();

    let mut components = Components::new();
    let mut component_registry = ComponentRegistry::new();

    let mut viewport = Viewport::new((80, 25));

    let mut floating = FloatingWidgets::empty();
    let mut glyph_map = GlyphMap::empty();

    let mut layout_ctx = LayoutCtx::new(
        variables,
        &factory,
        &mut states,
        &mut attribute_storage,
        &mut components,
        &mut component_registry,
        &mut floating,
        &mut glyph_map,
        &mut viewport,
        function_table,
        &doc.expressions,
    );

    let mut ctx = layout_ctx.eval_ctx(None, None);
    let scope = Scope::root();

    eval_blueprint(blueprint, &mut ctx, &scope, root_node(), &mut tree.view()).unwrap();

    let filter = crate::layout::LayoutFilter;
    let mut for_each = LayoutForEach::new(tree.view(), &scope, filter);
    _ = for_each
        .each(&mut layout_ctx, |ctx, widget, children| {
            _ = widget.layout(children, ctx.viewport.constraints(), ctx)?;
            Ok(ControlFlow::Break(()))
        })
        .unwrap();

    f(tree.view(), layout_ctx.attribute_storage);
}

// -----------------------------------------------------------------------------
//   - Test widgets -
// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct Many;

impl Widget for Many {
    fn layout<'bp>(
        &mut self,
        mut children: crate::LayoutForEach<'_, 'bp>,
        constraints: crate::layout::Constraints,
        _: crate::WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> crate::error::Result<anathema_geometry::Size> {
        let mut size = Size::ZERO;
        _ = children.each(ctx, |ctx, child, children| {
            let child_size: Size = child.layout(children, constraints, ctx)?.into();
            size.width = size.width.max(child_size.width);
            size.height += child_size.height;
            Ok(ControlFlow::Continue(()))
        });

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: crate::ForEach<'_, 'bp, crate::layout::PositionFilter>,
        _: crate::WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: crate::layout::PositionCtx,
    ) {
        _ = children.each(|child, children| {
            child.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Continue(())
        });
    }
}

#[derive(Debug, Default)]
pub(crate) struct Text(String);

impl Widget for Text {
    fn layout<'bp>(
        &mut self,
        _: crate::LayoutForEach<'_, 'bp>,
        _: crate::layout::Constraints,
        id: crate::WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> crate::error::Result<anathema_geometry::Size> {
        let attributes = ctx.attributes(id);
        let Some(value) = attributes.value() else { return Ok(Size::ZERO) };

        let mut buffer = String::new();
        value.strings(|s| write!(&mut buffer, "{s}").is_ok());
        self.0 = buffer;

        Ok(Size::new(self.0.chars().count() as u16, 1))
    }

    fn position<'bp>(
        &mut self,
        _: crate::ForEach<'_, 'bp, crate::layout::PositionFilter>,
        _: crate::WidgetId,
        _: &AttributeStorage<'bp>,
        _: crate::layout::PositionCtx,
    ) {
    }
}

#[derive(Debug, Default)]
pub(crate) struct Float;

impl Widget for Float {
    fn layout<'bp>(
        &mut self,
        mut children: crate::LayoutForEach<'_, 'bp>,
        constraints: crate::layout::Constraints,
        _: crate::WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> crate::error::Result<anathema_geometry::Size> {
        let mut size = Size::ZERO;
        _ = children.each(ctx, |ctx, child, children| {
            let child_size: Size = child.layout(children, constraints, ctx)?.into();
            size.width = size.width.max(child_size.width);
            size.height += child_size.height;
            Ok(ControlFlow::Continue(()))
        });

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: crate::ForEach<'_, 'bp, crate::layout::PositionFilter>,
        _: crate::WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: crate::layout::PositionCtx,
    ) {
        _ = children.each(|child, children| {
            child.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Continue(())
        });
    }

    fn floats(&self) -> bool {
        true
    }
}
