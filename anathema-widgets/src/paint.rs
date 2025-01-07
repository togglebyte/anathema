use std::ops::{ControlFlow, Deref};

use anathema_geometry::{LocalPos, Pos, Region, Size};
use anathema_state::{Color, Hex};
use anathema_store::indexmap::IndexMap;
use anathema_store::slab::SlabIndex;
use anathema_store::tree::{Node, TreeFilter, TreeForEach, TreeValues};
use anathema_strings::HStrings;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::layout::Display;
use crate::nodes::element::Element;
use crate::{AttributeStorage, ForEach, PaintChildren, WidgetContainer, WidgetId, WidgetKind};

pub type GlyphMap = IndexMap<GlyphIndex, String>;

pub struct Glyphs<'a> {
    inner: Graphemes<'a>,
}

impl<'a> Glyphs<'a> {
    pub fn new(src: &'a str) -> Self {
        let inner = src.graphemes(true);
        Self { inner }
    }

    pub fn next(&mut self, map: &mut GlyphMap) -> Option<Glyph> {
        let g = self.inner.next()?;
        let mut chars = g.chars();
        let c = chars.next()?;

        match chars.next() {
            None => Glyph::Single(c, c.width().unwrap_or(0) as u8),
            Some(_) => {
                let width = g.width();
                let glyph = map.insert(g.into());
                Glyph::Cluster(glyph, width as u8)
            }
        }
        .into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Glyph {
    Single(char, u8),
    Cluster(GlyphIndex, u8),
}

impl Glyph {
    pub fn space() -> Self {
        Self::Single(' ', 1)
    }

    pub fn is_newline(&self) -> bool {
        matches!(self, Self::Single('\n', _))
    }

    pub fn width(&self) -> usize {
        match self {
            Glyph::Single(_, width) | Glyph::Cluster(_, width) => *width as usize,
        }
    }

    pub const fn from_char(c: char, width: u8) -> Self {
        Self::Single(c, width)
    }
}

pub trait WidgetRenderer {
    fn draw_glyph(&mut self, glyph: Glyph, local_pos: Pos);

    fn set_attributes(&mut self, attribs: &dyn CellAttributes, local_pos: Pos);

    fn size(&self) -> Size;
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct GlyphIndex(u32);

impl SlabIndex for GlyphIndex {
    const MAX: usize = u32::MAX as usize;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u32)
    }
}

pub trait CellAttributes {
    fn with_str(&self, key: &str, f: &mut dyn FnMut(&str));

    fn get_i64(&self, key: &str) -> Option<i64>;

    fn get_u8(&self, key: &str) -> Option<u8>;

    fn get_hex(&self, key: &str) -> Option<Hex>;

    fn get_color(&self, key: &str) -> Option<Color>;

    fn get_bool(&self, key: &str) -> bool;
}

pub struct PaintFilter<'frame, 'bp> {
    attributes: &'frame AttributeStorage<'bp>,
    ignore_floats: bool,
}

impl<'frame, 'bp> PaintFilter<'frame, 'bp> {
    pub fn new(ignore_floats: bool, attributes: &'frame AttributeStorage<'bp>) -> Self {
        Self {
            attributes,
            ignore_floats,
        }
    }
}

impl<'frame, 'bp> TreeFilter for PaintFilter<'frame, 'bp> {
    type Input = WidgetContainer<'bp>;
    type Output = Element<'bp>;

    fn filter<'val>(
        &self,
        _widget_id: WidgetId,
        input: &'val mut Self::Input,
        _children: &[Node],
        _widgets: &mut TreeValues<WidgetContainer<'bp>>,
    ) -> ControlFlow<(), Option<&'val mut Self::Output>> {
        match &mut input.kind {
            WidgetKind::Element(el) if el.container.inner.any_floats() && self.ignore_floats => ControlFlow::Break(()),
            WidgetKind::Element(el) => {
                panic!("attributes needs to be combined with &HStrings for this to work");
                // match self
                //     .attributes
                //     .get(el.id())
                //     .get::<Display>("display")
                //     .unwrap_or_default()
                // {
                //     Display::Show => ControlFlow::Continue(Some(el)),
                //     Display::Hide | Display::Exclude => ControlFlow::Break(()),
                // }
            }
            // WidgetKind::If(widget) if !widget.show => ControlFlow::Break(()),
            // WidgetKind::Else(widget) if !widget.show => ControlFlow::Break(()),
            _ => ControlFlow::Continue(None),
        }
    }
}

// TODO: rename to paint filter and remove the old one
// TODO: filter out all exclude / hide widgets
pub struct PainFilter;

impl<'bp> crate::widget::Filter<'bp> for PainFilter {
    type Output = Element<'bp>;

    fn filter<'a>(widget: &'a mut WidgetContainer<'bp>) -> Option<&'a mut Self::Output> {
        match &mut widget.kind {
            WidgetKind::Element(element) => Some(element),
            _ => None,
        }
    }
}

pub fn paint<'bp>(
    surface: &mut impl WidgetRenderer,
    glyph_index: &mut GlyphMap,
    mut widgets: PaintChildren<'_, 'bp>,
    attribute_storage: &AttributeStorage<'bp>,
    strings: &HStrings<'bp>,
    ignore_floats: bool,
) {
    #[cfg(feature = "profile")]
    puffin::profile_function!();

    // let filter = PaintFilter::new(ignore_floats, attribute_storage);
    // let children = TreeForEach::new(children, values, &filter);
    widgets.each(|widget, children| {
        let ctx = PaintCtx::new(surface, None, glyph_index, strings);
        widget.paint(children, ctx, attribute_storage);
        ControlFlow::Continue(())
    });
    // element.paint(children, ctx, attribute_storage);
}

#[derive(Debug, Copy, Clone)]
pub struct Unsized;

// TODO rename this as it contains both size and position
pub struct SizePos {
    pub local_size: Size,
    pub global_pos: Pos,
}

impl SizePos {
    pub fn new(local_size: Size, global_pos: Pos) -> Self {
        Self { local_size, global_pos }
    }
}

// -----------------------------------------------------------------------------
//     - Paint context -
// -----------------------------------------------------------------------------
// * Context should draw in local coordinates and tranlate to the screen
// * A child always starts at 0, 0 in local space
/// Paint context used by the widgets to paint.
/// It works in local coordinates, translated to screen position.
pub struct PaintCtx<'surface, Size> {
    surface: &'surface mut dyn WidgetRenderer,
    pub clip: Option<Region>,
    pub(crate) state: Size,
    glyph_map: &'surface mut GlyphMap,
    pub strings: &'surface HStrings<'surface>,
}

impl<'surface> Deref for PaintCtx<'surface, SizePos> {
    type Target = SizePos;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'surface> PaintCtx<'surface, Unsized> {
    pub fn new(
        surface: &'surface mut dyn WidgetRenderer,
        clip: Option<Region>,
        glyph_map: &'surface mut GlyphMap,
        strings: &'surface HStrings<'_>,
    ) -> Self {
        Self {
            surface,
            clip,
            state: Unsized,
            glyph_map,
            strings,
        }
    }

    /// Create a sized context at a given position
    pub fn into_sized(self, size: Size, global_pos: Pos) -> PaintCtx<'surface, SizePos> {
        PaintCtx {
            surface: self.surface,
            glyph_map: self.glyph_map,
            clip: self.clip,
            state: SizePos::new(size, global_pos),
            strings: self.strings,
        }
    }
}

impl<'screen> PaintCtx<'screen, SizePos> {
    pub fn to_unsized(&mut self) -> PaintCtx<'_, Unsized> {
        PaintCtx::new(self.surface, self.clip, self.glyph_map, self.strings)
    }

    pub fn update(&mut self, new_size: Size, new_pos: Pos) {
        self.state.local_size = new_size;
        self.state.global_pos = new_pos;
    }

    /// This will create an intersection with any previous regions
    pub fn set_clip_region(&mut self, region: Region) {
        let current = self.clip.get_or_insert(region);
        *current = current.intersect_with(&region);
    }

    pub fn create_region(&self) -> Region {
        let mut region = Region::new(
            self.global_pos,
            Pos::new(
                self.global_pos.x + self.local_size.width as i32,
                self.global_pos.y + self.local_size.height as i32,
            ),
        );

        if let Some(existing) = self.clip {
            region.constrain(&existing);
        }

        region
    }

    fn clip(&self, local_pos: LocalPos, clip: &Region) -> bool {
        let pos = self.global_pos + local_pos;
        clip.contains(pos)
    }

    fn pos_inside_local_region(&self, pos: LocalPos, width: usize) -> bool {
        (pos.x as usize) + width <= self.local_size.width && (pos.y as usize) < self.local_size.height
    }

    // Translate local coordinates to screen coordinates.
    // Will return `None` if the coordinates are outside the screen bounds
    pub fn translate_to_global(&self, local: LocalPos) -> Option<Pos> {
        let screen_x = local.x as i32 + self.global_pos.x;
        let screen_y = local.y as i32 + self.global_pos.y;

        let (width, height) = self.surface.size().into();
        if screen_x < 0 || screen_y < 0 || screen_x >= width || screen_y >= height {
            return None;
        }

        Some(Pos {
            x: screen_x,
            y: screen_y,
        })
    }

    fn newline(&mut self, pos: LocalPos) -> Option<LocalPos> {
        let y = pos.y + 1; // next line
        if y as usize >= self.local_size.height {
            None
        } else {
            Some(LocalPos { x: 0, y })
        }
    }

    pub fn to_glyphs<'a>(&mut self, s: &'a str) -> Glyphs<'a> {
        Glyphs::new(s)
    }

    pub fn place_glyphs(&mut self, mut glyphs: Glyphs<'_>, mut pos: LocalPos) -> Option<LocalPos> {
        while let Some(glyph) = glyphs.next(self.glyph_map) {
            pos = self.place_glyph(glyph, pos)?;
        }
        Some(pos)
    }

    pub fn set_attributes(&mut self, attrs: &dyn CellAttributes, pos: LocalPos) {
        // Ensure that the position is inside provided clipping region
        if let Some(clip) = self.clip.as_ref() {
            if !self.clip(pos, clip) {
                return;
            }
        }

        let screen_pos = match self.translate_to_global(pos) {
            Some(pos) => pos,
            None => return,
        };

        self.surface.set_attributes(attrs, screen_pos);
    }

    // Place a char on the screen buffer, return the next cursor position in local space.
    //
    // The `input_pos` is the position, in local space, where the character
    // should be placed. This will (possibly) be offset if there is clipping available.
    //
    // The `output_pos` is the same as the `input_pos` unless clipping has been applied.
    pub fn place_glyph(&mut self, glyph: Glyph, input_pos: LocalPos) -> Option<LocalPos> {
        let width = glyph.width();
        let next = LocalPos {
            x: input_pos.x + width as u16,
            y: input_pos.y,
        };

        // Ensure that the position is inside provided clipping region
        if let Some(clip) = self.clip.as_ref() {
            if !self.clip(input_pos, clip) {
                return Some(next);
            }
        }

        // 1. Newline (yes / no)
        if glyph.is_newline() {
            return self.newline(input_pos);
        }

        // 2. Check if the char can be placed
        if !self.pos_inside_local_region(input_pos, width) {
            return None;
        }

        // 3. Find position on the screen
        let screen_pos = match self.translate_to_global(input_pos) {
            Some(pos) => pos,
            None => return Some(next),
        };

        // 4. Place the char
        self.surface.draw_glyph(glyph, screen_pos);

        // 4. Advance the cursor (which might trigger another newline)
        if input_pos.x >= self.local_size.width as u16 {
            self.newline(input_pos)
        } else {
            Some(LocalPos {
                x: input_pos.x + width as u16,
                y: input_pos.y,
            })
        }
    }
}
