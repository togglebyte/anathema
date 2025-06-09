use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_state::States;
use anathema_store::tree::root_node;
use anathema_templates::Document;
use anathema_value_resolver::{AttributeStorage, Attributes, Scope};

use crate::components::ComponentRegistry;
use crate::layout::{LayoutCtx, Viewport};
use crate::{eval_blueprint, Components, Factory, FloatingWidgets, GlyphMap, LayoutForEach, Widget, WidgetTree, WidgetTreeView};

pub struct TestCase {}

pub fn with_template<F>(tpl: &str, f: F)
where
    F: Fn(WidgetTreeView<'_, '_>),
{
    let mut tree = WidgetTree::empty();
    let mut doc = Document::new(tpl);
    let (blueprint, globals) = doc.compile().unwrap();
    let globals = Box::leak(Box::new(globals));
    let blueprint = Box::leak(Box::new(blueprint));

    let mut factory = Factory::new();
    factory.register_default::<Many>("many");
    factory.register_default::<Text>("text");

    let mut states = States::new();
    let mut attribute_storage = AttributeStorage::empty();

    let mut components = Components::new();
    let mut component_registry = ComponentRegistry::new();

    let mut viewport = Viewport::new((80, 25));

    let mut floating = FloatingWidgets::empty();
    let mut glyph_map = GlyphMap::empty();

    let mut layout_ctx = LayoutCtx::new(
        globals,
        &factory,
        &mut states,
        &mut attribute_storage,
        &mut components,
        &mut component_registry,
        &mut floating,
        &mut glyph_map,
        &mut viewport,
    );

    let mut ctx = layout_ctx.eval_ctx(None);
    let scope = Scope::root();

    eval_blueprint(blueprint, &mut ctx, &scope, root_node(), &mut tree.view_mut()).unwrap();

    let filter = crate::layout::LayoutFilter::fixed();
    let mut for_each = LayoutForEach::new(tree.view_mut(), &scope, filter, None);
    for_each.each(&mut layout_ctx, |ctx, widget, children| {
        _ = widget.layout(children, ctx.viewport.constraints(), ctx)?;
        Ok(ControlFlow::Break(()))
    }).unwrap();


    f(tree.view_mut());
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
        id: crate::WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> crate::error::Result<anathema_geometry::Size> {
        let mut size = Size::ZERO;
        children.each(ctx, |ctx, child, children| {
            let child_size: Size = child.layout(children, constraints, ctx)?.into();
            size.width = size.width.max(child_size.width);
            size.height += child_size.height;
            Ok(ControlFlow::Continue(()))
        });

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        children: crate::ForEach<'_, 'bp, crate::layout::PositionFilter>,
        id: crate::WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: crate::layout::PositionCtx,
    ) {
        // 
    }
}

#[derive(Debug, Default)]
pub(crate) struct Text;

impl Widget for Text {
    fn layout<'bp>(
        &mut self,
        children: crate::LayoutForEach<'_, 'bp>,
        constraints: crate::layout::Constraints,
        id: crate::WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> crate::error::Result<anathema_geometry::Size> {
        Ok(Size::ZERO)
    }

    fn position<'bp>(
        &mut self,
        children: crate::ForEach<'_, 'bp, crate::layout::PositionFilter>,
        id: crate::WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: crate::layout::PositionCtx,
    ) {
    }
}

