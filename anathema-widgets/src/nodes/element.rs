use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region, Size};
use anathema_value_resolver::AttributeStorage;

use crate::container::Container;
use crate::layout::{Constraints, LayoutCtx, PositionFilter, Viewport};
use crate::paint::{PaintCtx, PaintFilter, Unsized};
use crate::widget::{ForEach, PaintChildren, PositionChildren};
use crate::{LayoutChildren, LayoutForEach, WidgetId};

pub enum Layout {
    Changed(Size),
    Unchanged(Size),
}

impl From<Layout> for Size {
    fn from(value: Layout) -> Self {
        match value {
            Layout::Changed(size) | Layout::Unchanged(size) => size,
        }
    }
}

#[derive(Debug)]
pub struct Element<'bp> {
    pub ident: &'bp str,
    pub(crate) container: Container,
}

impl<'bp> Element<'bp> {
    pub fn id(&self) -> WidgetId {
        self.container.id
    }

    pub(crate) fn new(ident: &'bp str, container: Container) -> Self {
        Self {
            ident,
            container,
        }
    }

    pub fn layout(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Layout {
        // 1. Check cache
        // 2. Check cache of children
        //
        // If one of the children returns a `Changed` layout result
        // the transition the widget into full layout mode

        if let Some(size) = self.cached_size() {
            let mut rebuild = false;
            children.each(ctx, |ctx, node, children| {
                match node.layout(children, constraints, ctx) {
                    Layout::Changed(_) => {
                        rebuild = true;
                        return ControlFlow::Break(());
                    }
                    Layout::Unchanged(_) => return ControlFlow::Continue(()),
                }
            });

            if !rebuild {
                return Layout::Unchanged(size);
            }
        }

        crate::awful_debug!("rebuild for {}", self.ident);
        self.container.layout(children, constraints, ctx)
        // Layout::Changed(size)
    }

    pub fn invalidate_cache(&mut self) {
        self.container.cache.invalidate();
    }

    /// Position the element
    pub fn position(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
        viewport: Viewport,
    ) {
        self.container.position(children, pos, attribute_storage, viewport)
    }

    /// Draw an element to the surface
    pub fn paint(
        &mut self,
        children: ForEach<'_, 'bp, PaintFilter>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.container.paint(children, ctx, attribute_storage);
    }

    /// Return the cached size if the constraints are matching
    /// the cached constraints.
    pub fn cached_size(&self) -> Option<Size> {
        self.container.cache.size()
    }

    pub fn size(&self) -> Size {
        self.container.cache.size
    }

    /// Inner bounds in global space
    pub fn inner_bounds(&self) -> Region {
        self.container.inner_bounds
    }

    /// Get a mutable reference to the underlying widget of the given type
    ///
    /// # Panics
    ///
    /// Panics if the element is of a different type
    pub fn to<T: 'static>(&mut self) -> &mut T {
        self.try_to().expect("wrong element type")
    }

    /// Get a mutable reference to the underlying widget of the given type
    pub fn try_to<T: 'static>(&mut self) -> Option<&mut T> {
        self.container.inner.to_any_mut().downcast_mut::<T>()
    }

    /// Get a reference to the underlying widget of the given type
    ///
    /// # Panics
    ///
    /// Panics if hte element is of a different type
    pub fn to_ref<T: 'static>(&self) -> &T {
        self.try_to_ref().expect("wrong element type")
    }

    /// Get a reference to the underlying widget of the given type
    pub fn try_to_ref<T: 'static>(&self) -> Option<&T> {
        self.container.inner.to_any_ref().downcast_ref::<T>()
    }

    /// Get the position of the container
    pub fn get_pos(&self) -> Pos {
        self.container.pos
    }

    /// Returns true if the widget is a floating widget
    pub(crate) fn is_floating(&self) -> bool {
        self.container.inner.any_floats()
    }
}
