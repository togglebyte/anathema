use std::any::Any;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::ops::ControlFlow;

pub type WidgetId = anathema_store::slab::Key;

use anathema_geometry::{Pos, Rect, Size};
use anathema_state::StateId;
use anathema_store::slab::SecondaryMap;
use anathema_store::smallmap::SmallMap;
use anathema_store::sorted::SortedList;
use anathema_store::tree::{NodeWalker, Tree, TreeForEach};
use anathema_templates::WidgetComponentId;

pub use self::attributes::{AttributeStorage, Attributes};
pub use self::factory::Factory;
pub use self::query::Elements;
use crate::layout::{Constraints, LayoutCtx, LayoutFilter, PositionCtx};
use crate::paint::{CellAttributes, PaintCtx, PaintFilter, SizePos};
use crate::WidgetKind;

mod attributes;
mod factory;
mod query;

#[derive(Debug)]
pub struct CompEntry {
    pub widget_id: WidgetId,
    pub component_id: WidgetComponentId,
    pub state_id: StateId,
    path: Box<[u16]>,
}

impl PartialOrd for CompEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.path.cmp(&other.path))
    }
}

impl Ord for CompEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl Eq for CompEntry {}

impl PartialEq for CompEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

pub struct Components {
    pub tab_index: usize,
    inner: SortedList<CompEntry>,
    comp_ids: SmallMap<WidgetComponentId, usize>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            tab_index: 0,
            inner: SortedList::empty(),
            comp_ids: SmallMap::empty(),
        }
    }

    pub fn push(&mut self, path: Box<[u16]>, widget_id: WidgetId, state_id: StateId, component_id: WidgetComponentId) {
        let entry = CompEntry {
            path,
            widget_id,
            state_id,
            component_id,
        };
        self.comp_ids.set(component_id, self.inner.len());
        self.inner.push(entry);
    }

    pub fn remove(&mut self, path: &[u16]) {
        if let Some(index) = self.inner.binary_search_by(|entry| (*entry.path).cmp(path)) {
            let entry = self.inner.remove(index);
            self.comp_ids.remove(&entry.component_id);
        }
    }

    pub fn current(&mut self) -> Option<(WidgetId, StateId)> {
        self.get(self.tab_index)
    }

    pub fn get(&mut self, index: usize) -> Option<(WidgetId, StateId)> {
        self.inner.get(index).map(|e| (e.widget_id, e.state_id))
    }

    pub fn get_by_component_id(&mut self, id: WidgetComponentId) -> Option<&CompEntry> {
        let index = self.comp_ids.get(&id)?;
        self.inner.get(*index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CompEntry> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn dodgy_remove(&mut self, widget_id: WidgetId) {
        let Some(index) = self.inner.iter().position(|entry| entry.widget_id == widget_id) else { return };
        let entry = self.inner.remove(index);
        self.comp_ids.remove(&entry.component_id);
    }
}

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

pub struct DirtyWidgets {
    inner: Vec<WidgetId>,
}

impl DirtyWidgets {
    pub fn empty() -> Self {
        Self { inner: vec![] }
    }

    pub fn push(&mut self, widget_id: WidgetId) {
        self.inner.push(widget_id);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn apply(&self, tree: &mut Tree<WidgetKind<'_>>) {
        for id in &self.inner {
            let path = tree.path(*id);
            tree.apply_node_walker(&path, WidgetNeedsLayout);
        }
    }
}

struct WidgetNeedsLayout;

impl NodeWalker<WidgetKind<'_>> for WidgetNeedsLayout {
    fn apply(&mut self, widget: &mut WidgetKind<'_>) {
        if let WidgetKind::Element(el) = widget {
            el.container.needs_layout = true;
        }
    }
}

/// Parent in a component relationship
#[derive(Debug, Copy, Clone)]
pub struct Parent(pub WidgetComponentId);

impl From<Parent> for WidgetComponentId {
    fn from(value: Parent) -> Self {
        value.0
    }
}

impl From<WidgetComponentId> for Parent {
    fn from(value: WidgetComponentId) -> Self {
        Self(value)
    }
}

/// Component relationships, tracking the parent component of each component
pub struct ComponentParents(SecondaryMap<WidgetComponentId, Parent>);

impl ComponentParents {
    pub fn empty() -> Self {
        Self(SecondaryMap::empty())
    }

    pub fn try_remove(&mut self, key: WidgetComponentId) {
        self.0.try_remove(key);
    }

    pub fn get_parent(&self, child: WidgetComponentId) -> Option<Parent> {
        self.0.get(child).copied()
    }
}

pub type WidgetTree<'a> = Tree<WidgetKind<'a>>;
pub type LayoutChildren<'a, 'frame, 'bp> = TreeForEach<'a, 'frame, WidgetKind<'bp>, LayoutFilter<'frame, 'bp>>;
pub type PositionChildren<'a, 'frame, 'bp> = TreeForEach<'a, 'frame, WidgetKind<'bp>, LayoutFilter<'frame, 'bp>>;
pub type PaintChildren<'a, 'frame, 'bp> = TreeForEach<'a, 'frame, WidgetKind<'bp>, PaintFilter<'frame, 'bp>>;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum ValueKey<'bp> {
    #[default]
    Value,
    Attribute(&'bp str),
}

impl ValueKey<'_> {
    pub fn to_str(&self) -> &str {
        match self {
            ValueKey::Value => "value",
            ValueKey::Attribute(name) => name,
        }
    }
}

/// Any widget should never be implemented directly
/// as it's implemented for any type that implements `Widget`
pub trait AnyWidget {
    fn to_any_ref(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;

    fn any_layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size;

    fn any_position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    );

    fn any_paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PaintCtx<'_, SizePos>,
    );

    fn any_floats(&self) -> bool;

    fn any_inner_bounds(&self, pos: Pos, size: Size) -> Rect;

    fn any_needs_reflow(&self) -> bool;
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
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        self.layout(children, constraints, id, ctx)
    }

    fn any_position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.position(children, id, attribute_storage, ctx)
    }

    fn any_paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PaintCtx<'_, SizePos>,
    ) {
        self.paint(children, id, attribute_storage, ctx)
    }

    fn any_inner_bounds(&self, pos: Pos, size: Size) -> Rect {
        self.inner_bounds(pos, size)
    }

    fn any_floats(&self) -> bool {
        self.floats()
    }

    fn any_needs_reflow(&self) -> bool {
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
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size;

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, '_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        children.for_each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Continue(())
        });
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    );

    fn floats(&self) -> bool {
        false
    }

    fn inner_bounds(&self, pos: Pos, size: Size) -> Rect {
        Rect::from((pos, size))
    }

    fn needs_reflow(&self) -> bool {
        false
    }
}

impl Debug for dyn Widget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<dyn Widget>")
    }
}

pub trait WidgetRenderer {
    fn draw_glyph(&mut self, c: char, local_pos: Pos);

    fn set_attributes(&mut self, attribs: &dyn CellAttributes, local_pos: Pos);

    fn size(&self) -> Size;
}
