use anathema_geometry::{Pos, Region, Size};
use anathema_state::{State, StateId, States};
use anathema_templates::{ComponentBlueprintId, Globals};
use anathema_value_resolver::{AttributeStorage, Attributes, FunctionTable};
use display::DISPLAY;

pub use self::constraints::Constraints;
pub use self::display::Display;
use crate::components::{AnyComponent, ComponentKind, ComponentRegistry};
use crate::nodes::element::Element;
use crate::tree::{FilterOutput, WidgetPositionFilter};
use crate::{Components, Factory, FloatingWidgets, GlyphMap, WidgetContainer, WidgetId, WidgetKind};

mod constraints;
pub mod display;
pub mod text;

pub struct LayoutCtx<'frame, 'bp> {
    pub states: &'frame mut States,
    pub(super) globals: &'bp Globals,
    factory: &'frame Factory,
    pub attribute_storage: &'frame mut AttributeStorage<'bp>,
    pub components: &'frame mut Components,
    pub glyph_map: &'frame mut GlyphMap,
    pub viewport: &'frame mut Viewport,

    // Need these for the eval context
    pub floating_widgets: &'frame mut FloatingWidgets,
    pub component_registry: &'frame mut ComponentRegistry,
    pub new_components: Vec<(WidgetId, StateId)>,
    pub stop_runtime: bool,
    pub(super) function_table: &'bp FunctionTable,
}

impl<'frame, 'bp> LayoutCtx<'frame, 'bp> {
    pub fn new(
        globals: &'bp Globals,
        factory: &'frame Factory,
        states: &'frame mut States,
        attribute_storage: &'frame mut AttributeStorage<'bp>,
        components: &'frame mut Components,
        component_registry: &'frame mut ComponentRegistry,
        floating_widgets: &'frame mut FloatingWidgets,
        glyph_map: &'frame mut GlyphMap,
        viewport: &'frame mut Viewport,
        function_table: &'bp FunctionTable,
    ) -> Self {
        Self {
            states,
            attribute_storage,
            components,
            component_registry,
            globals,
            factory,
            floating_widgets,
            glyph_map,
            viewport,
            new_components: vec![],
            stop_runtime: false,
            function_table,
        }
    }

    pub fn attributes(&self, node_id: WidgetId) -> &Attributes<'bp> {
        self.attribute_storage.get(node_id)
    }

    pub fn eval_ctx(&mut self, parent_component: Option<WidgetId>) -> EvalCtx<'_, 'bp> {
        EvalCtx {
            floating_widgets: self.floating_widgets,
            attribute_storage: self.attribute_storage,
            states: self.states,
            component_registry: self.component_registry,
            components: self.components,
            globals: self.globals,
            factory: self.factory,
            parent_component,
            new_components: &mut self.new_components,
            function_table: self.function_table,
        }
    }
}

pub struct EvalCtx<'frame, 'bp> {
    pub(super) new_components: &'frame mut Vec<(WidgetId, StateId)>,
    pub(super) floating_widgets: &'frame mut FloatingWidgets,
    pub(super) attribute_storage: &'frame mut AttributeStorage<'bp>,
    pub(super) states: &'frame mut States,
    component_registry: &'frame mut ComponentRegistry,
    pub(super) components: &'frame mut Components,
    pub(super) globals: &'bp Globals,
    pub(super) factory: &'frame Factory,
    pub(super) function_table: &'bp FunctionTable,
    pub(super) parent_component: Option<WidgetId>,
}

impl<'frame, 'bp> EvalCtx<'frame, 'bp> {
    pub(super) fn get_component(
        &mut self,
        component_id: ComponentBlueprintId,
    ) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn State>)> {
        self.component_registry.get(component_id)
    }
}

#[derive(Debug, Copy, Clone)]
/// A viewport represents the available space in the root
pub struct Viewport {
    size: Size,
    region: Region,
}

impl Viewport {
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        let region = Region::from((Pos::ZERO, size));
        Self { size, region }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn region(&self) -> &Region {
        &self.region
    }

    pub fn constraints(&self) -> Constraints {
        Constraints::new(self.size.width, self.size.height)
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.region = Region::from((Pos::ZERO, size));
    }

    pub fn contains(&self, region: Region) -> bool {
        self.region.intersects(&region)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LayoutFilter;

impl<'bp> crate::widget::Filter<'bp> for LayoutFilter {
    type Output = WidgetContainer<'bp>;

    fn filter<'a>(
        &mut self,
        widget: &'a mut WidgetContainer<'bp>,
        attribute_storage: &AttributeStorage<'_>,
    ) -> FilterOutput<&'a mut Self::Output, Self> {
        match &mut widget.kind {
            WidgetKind::Element(element) => {
                let attributes = attribute_storage.get(element.id());
                match attributes.get_as::<Display>(DISPLAY).unwrap_or_default() {
                    Display::Show | Display::Hide => FilterOutput::Include(widget, *self),
                    Display::Exclude => FilterOutput::Exclude,
                }
            }
            _ => FilterOutput::Continue,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PositionCtx {
    pub inner_size: Size,
    pub pos: Pos,
    pub viewport: Viewport,
}

impl PositionCtx {
    pub fn region(&self) -> Region {
        Region::from((self.pos, self.inner_size))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PositionFilter(WidgetPositionFilter);

impl PositionFilter {
    pub fn fixed() -> Self {
        Self(WidgetPositionFilter::Fixed)
    }

    pub fn floating() -> Self {
        Self(WidgetPositionFilter::Floating)
    }

    pub fn all() -> Self {
        Self(WidgetPositionFilter::All)
    }

    pub fn none() -> Self {
        Self(WidgetPositionFilter::None)
    }
}

impl<'bp> crate::widget::Filter<'bp> for PositionFilter {
    type Output = Element<'bp>;

    fn filter<'a>(
        &mut self,
        widget: &'a mut WidgetContainer<'bp>,
        attribute_storage: &AttributeStorage<'_>,
    ) -> FilterOutput<&'a mut Self::Output, Self> {
        match &mut widget.kind {
            WidgetKind::Element(element) => {
                let attributes = attribute_storage.get(element.id());
                match attributes.get_as::<Display>(DISPLAY).unwrap_or_default() {
                    Display::Show => match self.0 {
                        WidgetPositionFilter::Floating => match element.is_floating() {
                            true => FilterOutput::Include(element, Self::all()),
                            false => FilterOutput::Continue,
                        },
                        WidgetPositionFilter::Fixed => match element.is_floating() {
                            false => FilterOutput::Include(element, *self),
                            true => FilterOutput::Include(element, Self::none()),
                        },
                        WidgetPositionFilter::All => FilterOutput::Include(element, *self),
                        WidgetPositionFilter::None => FilterOutput::Exclude,
                    },
                    Display::Hide | Display::Exclude => FilterOutput::Exclude,
                }
            }
            _ => FilterOutput::Continue,
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::ControlFlow;

    use super::*;
    use crate::widget::ForEach;

    #[test]
    fn filter_floating_positioning() {
        let tpl = "
        many
            many
                many
                // --------------------
                float                //
                    text             // <- only these
                        text         // <------------
                            float    //
                                text //
                // --------------------
                many
            many
                many
                    many
        ";

        let mut expected = vec!["float", "text", "text", "float", "text"];
        crate::testing::with_template(tpl, move |tree, attributes| {
            let filter = PositionFilter::floating();
            let children = ForEach::new(tree, attributes, filter);
            recur(children, &mut expected);
        });

        fn recur(mut f: ForEach<'_, '_, PositionFilter>, expected: &mut Vec<&'static str>) {
            _ = f.each(|el, nodes| {
                assert_eq!(expected.remove(0), el.ident);
                recur(nodes, expected);
                ControlFlow::Continue(())
            });
        }
    }
}
