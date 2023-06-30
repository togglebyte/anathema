use std::ops::Deref;

use anathema_render::{Screen, ScreenPos, Size, Style};
use unicode_width::UnicodeWidthChar;

pub use self::data::DataCtx;
use super::layout::{Constraints, Padding};
use super::{Align, LocalPos, Pos, Region};
use crate::gen::store::Store;
use crate::template::Template;
use crate::Lookup;

mod data;

// -----------------------------------------------------------------------------
//   - Layout -
// -----------------------------------------------------------------------------
#[derive(Copy, Clone)]
pub struct LayoutCtx<'widget, 'tpl, 'parent> {
    pub templates: &'tpl [Template],
    pub values: &'widget Store<'parent>,
    pub constraints: Constraints,
    pub padding: Padding,
    pub lookup: &'widget Lookup,
}

impl<'widget, 'tpl, 'parent> LayoutCtx<'widget, 'tpl, 'parent> {
    pub fn new(
        templates: &'tpl [Template],
        values: &'widget Store<'parent>,
        constraints: Constraints,
        padding: Padding,
        lookup: &'widget Lookup,
    ) -> Self {
        Self {
            templates,
            values,
            constraints,
            padding,
            lookup,
        }
    }

    pub fn padded_constraints(&self) -> Constraints {
        if self.padding != Padding::ZERO {
            let padding = self.padding;
            let mut constraints = self.constraints;

            constraints.max_width = constraints
                .max_width
                .saturating_sub(padding.left + padding.right);
            constraints.min_width = constraints.min_width.min(constraints.max_width);

            constraints.max_height = constraints
                .max_height
                .saturating_sub(padding.top + padding.bottom);
            constraints.min_height = constraints.min_height.min(constraints.max_height);

            constraints
        } else {
            self.constraints
        }
    }

    pub fn padding_size(&self) -> Size {
        if self.padding != Padding::ZERO {
            let padding = self.padding;
            Size::new(padding.left + padding.right, padding.top + padding.bottom)
        } else {
            Size::ZERO
        }
    }
}

// -----------------------------------------------------------------------------
//   - Paint context size -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct Unsized;

pub struct WithSize {
    pub local_size: Size,
    pub global_pos: Pos,
}

impl WithSize {
    pub fn new(local_size: Size, global_pos: Pos) -> Self {
        Self {
            local_size,
            global_pos,
        }
    }
}

// -----------------------------------------------------------------------------
//     - Paint context -
// -----------------------------------------------------------------------------
// * Context should draw in local coordinates and tranlate to the screen
// * A child always starts at 0, 0 in local space
/// Paint context used by the widgets to paint.
/// It works in local coordinates, translated to screen position.
pub struct PaintCtx<'screen, S> {
    screen: &'screen mut Screen,
    pub clip: Option<&'screen Region>,
    pub(crate) state: S,
}

impl<'screen> Deref for PaintCtx<'screen, WithSize> {
    type Target = WithSize;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'screen> PaintCtx<'screen, Unsized> {
    pub fn new(screen: &'screen mut Screen, clip: Option<&'screen Region>) -> Self {
        Self {
            screen,
            clip,
            state: Unsized,
        }
    }

    /// Create a sized context at a given position
    pub fn into_sized(self, size: Size, global_pos: Pos) -> PaintCtx<'screen, WithSize> {
        PaintCtx {
            screen: self.screen,
            clip: self.clip,
            state: WithSize::new(size, global_pos),
        }
    }
}

impl<'screen> PaintCtx<'screen, WithSize> {
    pub fn to_unsized(&mut self) -> PaintCtx<'_, Unsized> {
        PaintCtx::new(self.screen, self.clip)
    }

    pub fn update(&mut self, new_size: Size, new_pos: Pos) {
        self.state.local_size = new_size;
        self.state.global_pos = new_pos;
    }

    pub fn create_region(&self) -> Region {
        let mut region = Region::new(
            self.global_pos,
            Pos::new(
                self.global_pos.x + self.local_size.width as i32 - 1,
                self.global_pos.y + self.local_size.height as i32 - 1,
            ),
        );

        if let Some(existing) = self.clip {
            region.constrain(existing);
        }

        region
    }

    fn clip(&self, local_pos: LocalPos, clip: &Region) -> bool {
        let pos = self.global_pos + local_pos;
        clip.contains(pos)
    }

    fn pos_inside_local_region(&self, pos: LocalPos, width: usize) -> bool {
        pos.x + width <= self.local_size.width && pos.y < self.local_size.height
    }

    // Translate local coordinates to screen coordinates.
    // Will return `None` if the coordinates are outside the screen bounds
    fn translate_to_screen(&self, local: LocalPos) -> Option<ScreenPos> {
        let screen_x = local.x as i32 + self.global_pos.x;
        let screen_y = local.y as i32 + self.global_pos.y;

        if screen_x < 0
            || screen_y < 0
            || screen_x >= self.screen.size().width as i32
            || screen_y >= self.screen.size().height as i32
        {
            return None;
        }

        Some(ScreenPos {
            x: screen_x as u16,
            y: screen_y as u16,
        })
    }

    fn newline(&mut self, pos: LocalPos) -> Option<LocalPos> {
        let y = pos.y + 1; // next line
        if y >= self.local_size.height {
            None
        } else {
            Some(LocalPos { x: 0, y })
        }
    }

    pub fn print(&mut self, s: &str, style: Style, mut pos: LocalPos) -> Option<LocalPos> {
        for c in s.chars() {
            let p = self.put(c, style, pos)?;
            pos = p;
        }
        Some(pos)
    }

    // Place a char on the screen buffer, return the next cursor position in local space.
    //
    // The `input_pos` is the position, in local space, where the character
    // should be placed. This will (possibly) be offset if there is clipping available.
    //
    // The `outpout_pos` is the same as the `input_pos` unless clipping has been applied.
    pub fn put(&mut self, c: char, style: Style, input_pos: LocalPos) -> Option<LocalPos> {
        let width = c.width().unwrap_or(0);
        let next = LocalPos {
            x: input_pos.x + width,
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
        let screen_pos = match self.translate_to_screen(input_pos) {
            Some(pos) => pos,
            None => return Some(next),
        };
        self.screen.put(c, style, screen_pos);

        // 4. Advance the cursor (which might trigger another newline)
        if input_pos.x >= self.local_size.width {
            self.newline(input_pos)
        } else {
            Some(LocalPos {
                x: input_pos.x + width,
                y: input_pos.y,
            })
        }
    }

    pub fn sub_context<'a>(&'a mut self, clip: Option<&'a Region>) -> PaintCtx<'_, Unsized> {
        PaintCtx {
            screen: self.screen,
            clip,
            state: Unsized,
        }
    }
}

// // -----------------------------------------------------------------------------
// //     - Layout context -
// // -----------------------------------------------------------------------------
// pub struct LayoutCtx {
//     pub constraints: Constraints,
//     pub padding: Padding,
// }

// impl LayoutCtx {
//     pub fn new(constraints: Constraints, padding: Padding) -> Self {
//         Self {
//             constraints,
//             padding,
//         }
//     }

//     pub fn padding(&mut self) -> Padding {
//         self.padding.take()
//     }

//     pub fn padded_constraints(&self) -> Constraints {
//         if !self.padding.no_padding() {
//             let padding = self.padding;
//             let mut constraints = self.constraints;

//             constraints.max_width = constraints
//                 .max_width
//                 .saturating_sub(padding.left + padding.right);
//             constraints.min_width = constraints.min_width.min(constraints.max_width);

//             constraints.max_height = constraints
//                 .max_height
//                 .saturating_sub(padding.top + padding.bottom);
//             constraints.min_height = constraints.min_height.min(constraints.max_height);

//             constraints
//         } else {
//             self.constraints
//         }
//     }

//     pub fn padding_size(&self) -> Size {
//         if !self.padding.no_padding() {
//             let padding = self.padding;
//             Size::new(padding.left + padding.right, padding.top + padding.bottom)
//         } else {
//             Size::ZERO
//         }
//     }
// }

// -----------------------------------------------------------------------------
//     - Position context -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct PositionCtx {
    pub pos: Pos,
    pub inner_size: Size,
    pub alignment: Option<Align>,
}

impl PositionCtx {
    pub fn new(pos: Pos, inner_size: Size) -> Self {
        Self {
            pos,
            inner_size,
            alignment: None,
        }
    }
}

// pub struct UpdateCtx {
//     pub attributes: Attributes,
//     pub pos: Pos,
//     pub size: Size,
// }

// impl UpdateCtx {
//     pub fn new(attributes: Attributes, pos: Pos, size: Size) -> Self {
//         Self {
//             attributes,
//             pos,
//             size,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use anathema_render::Screen;

    use super::*;

    #[test]
    fn put() {
        // Put a character on screen
        let size = Size::new(10, 5);
        let mut screen = Screen::new(size);
        let global_pos = Pos::new(3, 2);
        let mut ctx = PaintCtx::new(&mut screen, None).into_sized(Size::new(2, 2), global_pos);

        ctx.put('x', Style::reset(), LocalPos::new(1, 1));

        let (actual, _) = screen.buffer().get(ScreenPos::new(4, 3)).unwrap();
        assert_eq!('x', actual);
    }

    #[test]
    fn clip() {
        // Put a character on screen
        let size = Size::new(25, 25);
        let mut screen = Screen::new(size);
        let global_pos = Pos::new(1, 1);
        let clipping_region = Region::new(global_pos, Pos::new(3, 3));
        let mut ctx = PaintCtx::new(&mut screen, Some(&clipping_region))
            .into_sized(Size::new(20, 20), global_pos);

        // Inside clipping space
        let first = LocalPos::new(1, 1);
        ctx.put('y', Style::reset(), first);

        // Outside clipping space
        let second = LocalPos::new(15, 15);
        ctx.put('z', Style::reset(), second);

        let index: ScreenPos = (first + global_pos).try_into().unwrap();
        let (actual, _) = screen.buffer().get(index).unwrap();
        assert_eq!('y', actual);

        let index: ScreenPos = (second + global_pos).try_into().unwrap();
        assert!(screen.buffer().get(index).is_none());
    }

    #[test]
    fn put_outside_of_screen() {
        // Unlike the `Screen` it self, trying to draw outside of the context
        // should not panic, just be ignored.
        //
        // Given a screen size of 1 x 1 and a paint context of 20 x 20
        // drawing outside of the 1 x 1 space should do nothing.
        let size = Size::new(1, 1);
        let mut screen = Screen::new(size);
        let mut ctx = PaintCtx::new(&mut screen, None).into_sized(Size::new(2, 2), Pos::ZERO);

        // Inside context, outside screen
        ctx.put('a', Style::reset(), LocalPos::new(2, 2));

        // Outside context
        ctx.put('b', Style::reset(), LocalPos::new(100, 100));

        assert!(screen.buffer().get(ScreenPos::new(2, 2)).is_none());
        assert!(screen.buffer().get(ScreenPos::new(100, 100)).is_none());
    }
}
