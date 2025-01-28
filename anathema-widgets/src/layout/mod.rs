use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region, Size};
use anathema_state::{AnyState, States, Subscriber};
use anathema_store::tree::{Node, TreeFilter, TreeForEach, TreeValues};
use anathema_strings::HStrings;
use anathema_templates::{ComponentBlueprintId, Globals};
use anathema_value_resolver::{AttributeStorage, Attributes, ResolverCtx, Scope};
use display::DISPLAY;

pub use self::constraints::Constraints;
pub use self::display::Display;
use crate::components::{AnyComponent, ComponentKind, ComponentRegistry};
use crate::nodes::element::Element;
use crate::tree::{FilterOutput, WidgetPositionFilter};
use crate::{
    ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, LayoutChildren, WidgetContainer,
    WidgetId, WidgetKind,
};

mod constraints;
pub mod display;
pub mod text;

pub struct LayoutCtx<'frame, 'bp> {
    pub states: &'frame mut States,
    pub(super) globals: &'bp Globals,
    pub dirty_widgets: &'frame mut DirtyWidgets,
    factory: &'frame Factory,
    pub attribute_storage: &'frame mut AttributeStorage<'bp>,
    pub components: &'frame mut Components,
    pub force_layout: bool,
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
        changelist: &'frame mut ChangeList,
        glyph_map: &'frame mut GlyphMap,
        dirty_widgets: &'frame mut DirtyWidgets,
        viewport: &'frame mut Viewport,
        force_layout: bool,
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
            dirty_widgets,
            viewport,
            force_layout,
        }
    }

    pub fn attributes(&self, node_id: WidgetId) -> &Attributes<'bp> {
        self.attribute_storage.get(node_id)
    }

    pub fn needs_layout(&self, node_id: WidgetId) -> bool {
        self.dirty_widgets.contains(node_id) || self.force_layout
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
                        WidgetPositionFilter::Floating => FilterOutput::Include(widget, *self),
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
                    Display::Show | Display::Hide => match self.0 {
                        WidgetPositionFilter::Floating => match element.is_floating() {
                            true => FilterOutput::Include(element, PositionFilter::fixed()),
                            false => FilterOutput::Continue,
                        },
                        WidgetPositionFilter::Fixed => match element.is_floating() {
                            false => FilterOutput::Include(element, *self),
                            true => FilterOutput::Exclude,
                        },
                    },
                    Display::Exclude => FilterOutput::Exclude,
                }
            }
            _ => FilterOutput::Continue,
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_geometry::Region;

    use super::*;

    #[test]
    fn region_inersect() {
        let a = Region::new(Pos::ZERO, Pos::new(10, 10));
        let b = Region::new(Pos::new(5, 5), Pos::new(8, 8));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn region_contains() {
        let a = Region::new(Pos::ZERO, Pos::new(10, 10));
        assert!(a.contains(Pos::ZERO));
        assert!(a.contains(Pos::new(9, 9)));
        assert!(!a.contains(Pos::new(10, 10)));
    }
}
