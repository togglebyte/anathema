use std::any::Any;
use std::fmt::{self, Debug};
use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region, Size};
use anathema_state::StateId;
use anathema_store::slab::SecondaryMap;
use anathema_store::tree::{Tree, TreeView};
use anathema_templates::ComponentBlueprintId;
use anathema_value_resolver::AttributeStorage;

pub use self::factory::Factory;
pub use self::style::{Attributes, Style};
use crate::WidgetContainer;
use crate::error::Result;
use crate::layout::{Constraints, LayoutCtx, PositionCtx, PositionFilter};
use crate::paint::{PaintCtx, PaintFilter, SizePos};
pub use crate::tree::{Filter, ForEach, LayoutForEach};

mod factory;
mod style;

pub type WidgetTreeView<'a, 'bp> = TreeView<'a, WidgetContainer<'bp>>;
pub type WidgetTree<'a> = Tree<WidgetContainer<'a>>;
pub type LayoutChildren<'a, 'bp> = LayoutForEach<'a, 'bp>;
pub type PositionChildren<'a, 'bp> = ForEach<'a, 'bp, PositionFilter>;
pub type PaintChildren<'a, 'bp> = ForEach<'a, 'bp, PaintFilter>;
pub type WidgetId = anathema_store::slab::Key;

#[derive(Debug)]
pub struct CompEntry {
    /// The state owned by this component
    pub state_id: StateId,
    /// The components id in the widget tree
    pub widget_id: WidgetId,

    /// Does the component accept tick events
    pub accept_ticks: bool,

    component_id: ComponentBlueprintId,
}

// -----------------------------------------------------------------------------
//   - Components -
// -----------------------------------------------------------------------------

/// Store a list of components currently in the tree
pub struct Components {
    inner: Vec<CompEntry>,
}

impl Components {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn push(
        &mut self,
        component_id: ComponentBlueprintId,
        widget_id: WidgetId,
        state_id: StateId,
        accept_ticks: bool,
    ) {
        let entry = CompEntry {
            component_id,
            widget_id,
            state_id,
            accept_ticks,
        };

        self.inner.push(entry)
    }

    pub fn try_remove(&mut self, widget_id: WidgetId) {
        self.inner.retain(|entry| entry.widget_id != widget_id);
    }

    /// Get the component by its index
    pub fn get(&mut self, index: usize) -> Option<(WidgetId, StateId)> {
        self.inner.get(index).map(|e| (e.widget_id, e.state_id))
    }

    /// This is used to send messages to components.
    /// The `ComponentBlueprintId` is only available to components that were added
    /// as a singular component, not prototypes
    pub fn get_by_component_id(&mut self, id: ComponentBlueprintId) -> Option<&CompEntry> {
        self.inner.iter().find(|e| e.component_id == id)
    }

    /// Get the component by its widget id
    pub fn get_by_widget_id(&mut self, id: WidgetId) -> Option<(WidgetId, StateId)> {
        self.inner
            .iter()
            .find(|entry| entry.widget_id == id)
            .map(|e| (e.widget_id, e.state_id))
    }

    /// Get widget id and state id for a component that accepts tick events
    pub fn get_ticking(&self, index: usize) -> Option<(WidgetId, StateId)> {
        self.inner
            .get(index)
            .and_then(|e| e.accept_ticks.then_some((e.widget_id, e.state_id)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &CompEntry> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug)]
pub struct FloatingWidgets(SecondaryMap<WidgetId, WidgetId>);

impl FloatingWidgets {
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    pub fn try_remove(&mut self, key: WidgetId) {
        self.0.try_remove(key);
    }

    pub(crate) fn insert(&mut self, widget_id: WidgetId) {
        self.0.insert(widget_id, widget_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WidgetId> {
        self.0.iter()
    }
}

/// Parent in a component relationship
#[derive(Debug, Copy, Clone)]
pub struct Parent(pub WidgetId);

impl From<Parent> for WidgetId {
    fn from(value: Parent) -> Self {
        value.0
    }
}

impl From<WidgetId> for Parent {
    fn from(value: WidgetId) -> Self {
        Self(value)
    }
}

/// Component relationships, tracking the parent component of each component
pub struct ComponentParents(SecondaryMap<ComponentBlueprintId, Parent>);

impl ComponentParents {
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    pub fn try_remove(&mut self, key: ComponentBlueprintId) {
        self.0.try_remove(key);
    }

    pub fn get_parent(&self, child: ComponentBlueprintId) -> Option<Parent> {
        self.0.get(child).copied()
    }
}

// -----------------------------------------------------------------------------
//   - Any widget -
// -----------------------------------------------------------------------------

/// Any widget should never be implemented directly
/// as it's implemented for any type that implements `Widget`
pub trait AnyWidget {
    fn to_any_ref(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn any_layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size>;

    fn any_position<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    );

    fn any_paint<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PaintFilter>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PaintCtx<'_, SizePos>,
    );

    fn any_floats(&self) -> bool;

    fn any_inner_bounds(&self, pos: Pos, size: Size) -> Region;

    fn any_needs_reflow(&mut self) -> bool;
}

impl<T: 'static + Widget> AnyWidget for T {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn any_layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        self.layout(children, constraints, id, ctx)
    }

    fn any_position<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.position(children, id, attribute_storage, ctx)
    }

    fn any_paint<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PaintFilter>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PaintCtx<'_, SizePos>,
    ) {
        self.paint(children, id, attribute_storage, ctx)
    }

    fn any_inner_bounds(&self, pos: Pos, size: Size) -> Region {
        self.inner_bounds(pos, size)
    }

    fn any_floats(&self) -> bool {
        self.floats()
    }

    fn any_needs_reflow(&mut self) -> bool {
        self.needs_reflow()
    }
}

impl Debug for dyn AnyWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<dyn AnyWidget>")
    }
}

pub trait Widget {
    fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size>;

    fn paint<'bp>(
        &mut self,
        mut children: ForEach<'_, 'bp, PaintFilter>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        _ = children.each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Continue(())
        });
    }

    fn position<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    );

    fn floats(&self) -> bool {
        false
    }

    fn inner_bounds(&self, pos: Pos, size: Size) -> Region {
        Region::from((pos, size))
    }

    fn needs_reflow(&mut self) -> bool {
        false
    }
}

impl Debug for dyn Widget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<dyn Widget>")
    }
}

// -----------------------------------------------------------------------------
//   - Dirty widgets collection -
// -----------------------------------------------------------------------------
pub struct DirtyWidgets {
    pub inner: Vec<WidgetId>,
    last_id: Option<WidgetId>,
}

impl DirtyWidgets {
    pub fn empty() -> Self {
        Self {
            inner: vec![],
            last_id: None,
        }
    }

    pub fn push(&mut self, widget_id: WidgetId) {
        if let Some(last) = self.last_id
            && last == widget_id
        {
            return;
        }

        self.last_id.replace(widget_id);
        self.inner.push(widget_id);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn drain(&mut self) -> impl Iterator<Item = WidgetId> {
        _ = self.last_id.take();
        self.inner.drain(..)
    }
}
