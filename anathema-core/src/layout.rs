//! Layout calculation and storage for UI elements.
//!
//! This module provides the infrastructure for storing calculated layout information
//! for each element in the UI tree. The actual layout calculation is performed by
//! widgets during the layout phase, while this module stores the results.
//!
//! # Overview
//!
//! The layout system manages:
//!
//! - **Position**: Where each element is located in screen coordinates
//! - **Size**: How large each element is (width and height)
//! - **Region**: The combined position and size (rectangular area) for each element
//!
//! # Layout Process
//!
//! The layout process in Anathema follows these steps:
//!
//! 1. **Initialization**: Each element is inserted into the layout with a zero region
//! 2. **Widget Layout**: Widgets calculate their size based on constraints and children
//! 3. **Size Storage**: Calculated sizes are stored in the layout system
//! 4. **Positioning**: Parent widgets set child positions based on their layout strategy
//! 5. **Position Storage**: Final positions are stored in the layout system
//!
//! # Coordinate System
//!
//! Positions are specified in screen coordinates:
//!
//! - Origin (0, 0) is at the top-left corner
//! - X increases to the right
//! - Y increases downward
//!
//! # Regions
//!
//! A [`Region`] combines position and size into a single rectangular area. It represents
//! the complete layout information for an element - where it starts and how much space
//! it occupies.
//!
//! # Usage
//!
//! The [`Layout`] type is primarily used internally by the runtime and widget system.
//! Widgets receive a mutable reference to the layout during their `layout` method,
//! allowing them to query and update layout information for their children.
//!
//! # Example
//!
//! ```rust,ignore
//! use anathema_core::layout::Layout;
//! use anathema_geometry::{Pos, Size};
//!
//! let mut layout = Layout::empty();
//!
//! // Insert an element
//! layout.insert(element_id);
//!
//! // Set the element's size after calculation
//! layout.set_size(element_id, Size::new(80, 24));
//!
//! // Set the element's position
//! layout.set_pos(element_id, Pos::new(10, 5));
//! ```
//!
//! # Performance
//!
//! Layout information is stored in a [`SecondaryMap`] keyed by [`ElementId`], providing
//! O(1) access to layout information for any element. The secondary map structure is
//! efficient for sparse mappings where not all possible IDs are used.

use anathema_geometry::{Pos, Region, Size};
use anathema_store::slab::SecondaryMap;

use crate::runtime::elements::ElementId;

/// Storage for calculated layout information for all elements.
///
/// This type maintains a mapping from element IDs to their calculated layout regions.
/// Each region contains both the position and size of an element, representing the
/// complete layout information needed for rendering.
///
/// # Structure
///
/// Internally, the layout uses a [`SecondaryMap`] indexed by [`ElementId`], allowing
/// efficient O(1) access to any element's layout information. The secondary map is
/// well-suited for sparse ID spaces where not all possible IDs are in use.
///
/// # Usage
///
/// Layout information is typically managed by the runtime system:
///
/// 1. Elements are inserted with zero regions when created
/// 2. During layout, widgets calculate sizes and store them
/// 3. During positioning, parent widgets set child positions
/// 4. During rendering, widgets read their layout information
///
/// # Example
///
/// ```rust,ignore
/// use anathema_core::layout::Layout;
/// use anathema_geometry::{Pos, Size};
///
/// let mut layout = Layout::empty();
///
/// // Insert an element
/// layout.insert(element_id);
///
/// // Widget calculates and stores its size
/// let size = Size::new(100, 50);
/// layout.set_size(element_id, size);
///
/// // Parent widget sets the element's position
/// let pos = Pos::new(10, 20);
/// layout.set_pos(element_id, pos);
/// ```
#[derive(Debug)]
pub struct Layout {
    /// Mapping from element IDs to their layout regions
    regions: SecondaryMap<ElementId, Region>,
}

impl Layout {
    /// Create a new empty layout storage.
    ///
    /// The layout starts with no elements. Elements must be explicitly inserted
    /// before their layout information can be accessed or modified.
    ///
    /// # Returns
    ///
    /// A new `Layout` instance with no elements.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_core::layout::Layout;
    ///
    /// let layout = Layout::empty();
    /// ```
    pub(crate) fn empty() -> Self {
        Self {
            regions: SecondaryMap::empty(),
        }
    }

    /// Update the size of an element's region.
    ///
    /// This method resizes the element's region while maintaining its position.
    /// It is typically called after a widget calculates its layout size.
    ///
    /// # Parameters
    ///
    /// - `id`: The element ID whose size should be updated
    /// - `size`: The new size for the element
    ///
    /// # Panics
    ///
    /// Panics if the element ID has not been inserted into the layout.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_geometry::Size;
    ///
    /// // Widget calculates its size
    /// let size = Size::new(80, 24);
    /// layout.set_size(element_id, size);
    /// ```
    pub(crate) fn set_size(&mut self, id: ElementId, size: Size) {
        self.regions[id].resize(size);
    }

    /// Update the position of an element's region.
    ///
    /// This method moves the element's region to a new position while maintaining
    /// its size. It is typically called by parent widgets when positioning their
    /// children according to their layout strategy.
    ///
    /// # Parameters
    ///
    /// - `id`: The element ID whose position should be updated
    /// - `pos`: The new position for the element
    ///
    /// # Panics
    ///
    /// Panics if the element ID has not been inserted into the layout.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_geometry::Pos;
    ///
    /// // Parent positions child at (10, 5)
    /// let pos = Pos::new(10, 5);
    /// layout.set_pos(child_id, pos);
    /// ```
    pub(crate) fn set_pos(&mut self, id: ElementId, pos: Pos) {
        self.regions[id].set_pos(pos);
    }

    /// Insert a new element into the layout with a zero region.
    ///
    /// This method initializes an element's layout information with a position of
    /// (0, 0) and a size of (0, 0). The actual position and size should be set
    /// during the layout phase.
    ///
    /// # Parameters
    ///
    /// - `id`: The element ID to insert
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Insert a new element when it's created
    /// layout.insert(element_id);
    ///
    /// // Later, set its actual size and position
    /// layout.set_size(element_id, calculated_size);
    /// layout.set_pos(element_id, calculated_pos);
    /// ```
    pub(crate) fn insert(&mut self, id: ElementId) {
        self.regions.insert(id, Region::ZERO);
    }

    /// Iterate over all layout regions.
    ///
    /// Returns an iterator over all stored regions, in an unspecified order.
    /// This is useful for debugging or inspecting the entire layout state.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to [`Region`] values.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Iterate over all regions
    /// for region in layout.iter() {
    ///     println!("Region: {:?}", region);
    /// }
    /// ```
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Region> {
        self.regions.iter()
    }
}
