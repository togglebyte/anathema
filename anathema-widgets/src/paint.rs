use std::ops::{ControlFlow, Deref};

use anathema_geometry::{LocalPos, Pos, Region, Size};
use anathema_state::{Color, Hex};
use anathema_store::tree::{Node, TreeFilter, TreeForEach, TreeValues};
use unicode_width::UnicodeWidthChar;

use crate::layout::Display;
use crate::nodes::element::Element;
use crate::widget::WidgetRenderer;
use crate::{AttributeStorage, WidgetId, WidgetKind};

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
    type Input = WidgetKind<'bp>;
    type Output = Element<'bp>;

    fn filter<'val>(
        &self,
        _widget_id: WidgetId,
        input: &'val mut Self::Input,
        _children: &[Node],
        _widgets: &mut TreeValues<WidgetKind<'bp>>,
    ) -> ControlFlow<(), Option<&'val mut Self::Output>> {
        match input {
            WidgetKind::Element(el) if el.container.inner.any_floats() && self.ignore_floats => ControlFlow::Break(()),
            WidgetKind::Element(el) => match self
                .attributes
                .get(el.id())
                .get::<Display>("display")
                .unwrap_or_default()
            {
                Display::Show => ControlFlow::Continue(Some(el)),
                Display::Hide | Display::Exclude => ControlFlow::Continue(None),
            },
            WidgetKind::If(widget) if !widget.show => ControlFlow::Break(()),
            WidgetKind::Else(widget) if !widget.show => ControlFlow::Break(()),
            _ => ControlFlow::Continue(None),
        }
    }
}

pub fn paint<'bp>(
    surface: &mut impl WidgetRenderer,
    element: &mut Element<'bp>,
    children: &[Node],
    values: &mut TreeValues<WidgetKind<'bp>>,
    attribute_storage: &AttributeStorage<'bp>,
    ignore_floats: bool,
) {
    let filter = PaintFilter::new(ignore_floats, attribute_storage);
    let children = TreeForEach::new(children, values, &filter);
    let ctx = PaintCtx::new(surface, None);
    element.paint(children, ctx, attribute_storage);
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
}

impl<'surface> Deref for PaintCtx<'surface, SizePos> {
    type Target = SizePos;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'surface> PaintCtx<'surface, Unsized> {
    pub fn new(surface: &'surface mut dyn WidgetRenderer, clip: Option<Region>) -> Self {
        Self {
            surface,
            clip,
            state: Unsized,
        }
    }

    /// Create a sized context at a given position
    pub fn into_sized(self, size: Size, global_pos: Pos) -> PaintCtx<'surface, SizePos> {
        PaintCtx {
            surface: self.surface,
            clip: self.clip,
            state: SizePos::new(size, global_pos),
        }
    }
}

impl<'screen> PaintCtx<'screen, SizePos> {
    pub fn to_unsized(&mut self) -> PaintCtx<'_, Unsized> {
        PaintCtx::new(self.surface, self.clip)
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

    pub fn place_glyphs(&mut self, s: &str, mut pos: LocalPos) -> Option<LocalPos> {
        for c in s.chars() {
            let p = self.place_glyph(c, pos)?;
            pos = p;
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
    pub fn place_glyph(&mut self, c: char, input_pos: LocalPos) -> Option<LocalPos> {
        let width = c.width().unwrap_or(0);
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
        if c == '\n' {
            return self.newline(input_pos);
        }

        // 2. Check if the char can be placed
        if !self.pos_inside_local_region(input_pos, width) {
            return None;
        }

        // 3. Place the char
        let screen_pos = match self.translate_to_global(input_pos) {
            Some(pos) => pos,
            None => return Some(next),
        };
        self.surface.draw_glyph(c, screen_pos);

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
