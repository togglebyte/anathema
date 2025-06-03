use anathema_geometry::{Pos, Region, Size};
use anathema_state::{AnyState, States};
use anathema_templates::{ComponentBlueprintId, Globals};
use anathema_value_resolver::{AttributeStorage, Attributes};
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
        }
    }

    pub fn attributes(&self, node_id: WidgetId) -> &Attributes<'bp> {
        self.attribute_storage.get(node_id)
    }

    pub fn eval_ctx(&mut self, parent_component: Option<WidgetId>) -> EvalCtx<'_, 'bp> {
        EvalCtx {
            floating_widgets: self.floating_widgets,
            attribute_storage: self.attribute_storage,
            states: &mut self.states,
            component_registry: self.component_registry,
            components: self.components,
            globals: self.globals,
            factory: &self.factory,
            parent_component,
        }
    }
}

pub struct EvalCtx<'frame, 'bp> {
    pub(super) floating_widgets: &'frame mut FloatingWidgets,
    pub(super) attribute_storage: &'frame mut AttributeStorage<'bp>,
    pub(super) states: &'frame mut States,
    component_registry: &'frame mut ComponentRegistry,
    pub(super) components: &'frame mut Components,
    pub(super) globals: &'bp Globals,
    pub(super) factory: &'frame Factory,
    pub(super) parent_component: Option<WidgetId>,
}

impl<'frame, 'bp> EvalCtx<'frame, 'bp> {
    pub(super) fn get_component(
        &mut self,
        component_id: ComponentBlueprintId,
    ) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn AnyState>)> {
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
pub struct LayoutFilter(WidgetPositionFilter);

impl LayoutFilter {
    pub fn fixed() -> Self {
        Self(WidgetPositionFilter::Fixed)
    }

    pub fn floating() -> Self {
        Self(WidgetPositionFilter::Floating)
    }
}

impl<'bp> crate::widget::Filter<'bp> for LayoutFilter {
    type Output = WidgetContainer<'bp>;

    fn filter<'a>(
        &self,
        widget: &'a mut WidgetContainer<'bp>,
        attribute_storage: &AttributeStorage<'_>,
    ) -> FilterOutput<&'a mut Self::Output, Self> {
        match &mut widget.kind {
            WidgetKind::Element(element) => {
                let attributes = attribute_storage.get(element.id());
                match attributes.get_as::<Display>(DISPLAY).unwrap_or_default() {
                    Display::Show | Display::Hide => match self.0 {
                        // Floating
                        WidgetPositionFilter::Floating => FilterOutput::Include(widget, *self),

                        // Fixed
                        WidgetPositionFilter::Fixed if !element.is_floating() => FilterOutput::Include(widget, *self),
                        _ => FilterOutput::Exclude,
                    },
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
}

impl<'bp> crate::widget::Filter<'bp> for PositionFilter {
    type Output = Element<'bp>;

    fn filter<'a>(
        &self,
        widget: &'a mut WidgetContainer<'bp>,
        attribute_storage: &AttributeStorage<'_>,
    ) -> FilterOutput<&'a mut Self::Output, Self> {
        match &mut widget.kind {
            // If this is the floating widget step then once a floating widget is found
            // the filter should change for the children of the floating widget to be a fixed
            // filter instead.
            WidgetKind::Element(element) => {
                let attributes = attribute_storage.get(element.id());
                match attributes.get_as::<Display>(DISPLAY).unwrap_or_default() {
                    Display::Show | Display::Hide => FilterOutput::Include(element, *self),
                    Display::Exclude => FilterOutput::Exclude,
                }
            }
            _ => FilterOutput::Continue,
        }
    }
}
