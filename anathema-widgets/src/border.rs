use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, Layout, Widget};
use anathema_runtime::{Attributes, Constraints, TemplateValue, WidgetAttributes};
use compact_str::CompactString;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const CORNER_COUNT: usize = 8;

const CORNER_TL: usize = 0;
const CORNER_T: usize = 1;
const CORNER_TR: usize = 2;
const CORNER_R: usize = 3;
const CORNER_BR: usize = 4;
const CORNER_B: usize = 5;
const CORNER_BL: usize = 6;
const CORNER_L: usize = 7;

const DEFAULT_SLIM_EDGES: CompactString = CompactString::const_new("┌─┐│┘─└│");

type Index = u8;

struct BorderRep<'a> {
    top_left: &'a str,
    top: &'a str,
    top_right: &'a str,
    right: &'a str,
    bottom_right: &'a str,
    bottom: &'a str,
    bottom_left: &'a str,
    left: &'a str,
}

#[derive(Debug, Default)]
struct BorderBusiness {
    style: CompactString,
    left_width: u8,
    right_width: u8,
    corners: [Index; CORNER_COUNT],
}

impl BorderBusiness {
    const fn empty() -> Self {
        Self {
            style: DEFAULT_SLIM_EDGES,
            left_width: 1,
            right_width: 1,
            corners: [0, 3, 6, 9, 12, 15, 18, 21],
        }
    }

    fn size(&self) -> Size {
        let width = self.left_width + self.right_width;
        Size::new(width as u32, 2)
    }

    fn fix_if_expired(&mut self, border_style: &str) {
        if border_style == self.style {
            return;
        }

        let graphemes = self.style.grapheme_indices(true);
        let mut left = 1;
        let mut right = 1;

        for (corner_id, (idx, edge)) in graphemes.take(CORNER_COUNT).enumerate() {
            match corner_id {
                CORNER_TL => {
                    left = left.max(edge.width() as u32);
                    self.corners[CORNER_TL] = idx as u8;
                }
                CORNER_BL => {
                    left = left.max(edge.width() as u32);
                    self.corners[CORNER_BL] = idx as u8;
                }
                CORNER_L => {
                    left = left.max(edge.width() as u32);
                    self.corners[CORNER_L] = idx as u8;
                }
                CORNER_TR => {
                    right = right.max(edge.width() as u32);
                    self.corners[CORNER_TR] = idx as u8;
                }
                CORNER_R => {
                    right = right.max(edge.width() as u32);
                    self.corners[CORNER_R] = idx as u8;
                }
                CORNER_BR => {
                    right = right.max(edge.width() as u32);
                    self.corners[CORNER_BR] = idx as u8;
                }
                CORNER_T => self.corners[CORNER_T] = idx as u8,
                CORNER_B => self.corners[CORNER_B] = idx as u8,
                _ => unreachable!(),
            }
        }

        self.left_width = left as u8;
        self.right_width = right as u8;
    }

    fn slice(&self, from: usize, to: usize) -> &str {
        let from = self.corners[from] as usize;
        let to = self.corners[to] as usize;
        &self.style[from..to]
    }

    fn tl(&self) -> &str {
        self.slice(CORNER_TL, CORNER_T)
    }

    fn t(&self) -> &str {
        self.slice(CORNER_T, CORNER_TR)
    }

    fn tr(&self) -> &str {
        self.slice(CORNER_TR, CORNER_R)
    }

    fn r(&self) -> &str {
        self.slice(CORNER_R, CORNER_BR)
    }

    fn br(&self) -> &str {
        self.slice(CORNER_BR, CORNER_B)
    }

    fn b(&self) -> &str {
        self.slice(CORNER_B, CORNER_BL)
    }

    fn bl(&self) -> &str {
        self.slice(CORNER_BL, CORNER_L)
    }

    fn l(&self) -> &str {
        let from = self.corners[CORNER_L] as usize;
        let to = self.style.len();
        &self.style[from..to]
    }
}

#[derive(Debug, Default)]
pub struct Border {
    border: BorderBusiness,
}

impl Border {
    pub(crate) fn new<'bp>(attrs: &Attributes<'bp>) -> Self {
        Self {
            border: BorderBusiness::empty(),
        }
    }
}

impl<'bp> Widget<'bp> for Border {
    fn layout(
        &mut self,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layout,
        mut constraints: Constraints,
    ) -> Size {
        // 1. Check if the border style has changed
        if let Some(border_style) = attributes.get_as::<&str>("border_style") {
            self.border.fix_if_expired(border_style);
        }

        // 2. Shrink constraint by border size
        let border_size = self.border.size();

        if let Some(min_width) = attributes.get_as::<u32>("min_width") {
            constraints.min.width = constraints.min.width.max(min_width);
        }

        if let Some(min_height) = attributes.get_as::<u32>("min_heigth") {
            constraints.min.height = constraints.min.height.max(min_height);
        }

        if let Some(width) = attributes.get_as::<u32>("width") {
            constraints.try_fit_width(width);
        }

        if let Some(height) = attributes.get_as::<u32>("height") {
            constraints.try_fit_height(height);
        }

        // 3. Layout children with the new constraint
        constraints -= border_size;
        let mut size = match children.next().map(|child| child.layout(layout, constraints)) {
            Some(size) => size + border_size,
            None => border_size,
        };

        // 4. If a fixed width / height is set, apply this to the output size
        if let Some(width) = attributes.get_as::<u32>("width") {
            size.width = width;
        }

        if let Some(height) = attributes.get_as::<u32>("height") {
            size.height = height;
        }

        size
    }

    fn position(
        &mut self,
        mut children: Children<'_, '_>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layout,
        mut pos: Pos,
    ) {
        if let Some(child) = children.next() {
            pos.x += self.border.left_width as i32;
            pos.y += 1;
            child.position(layout, pos);
        }
    }

    fn paint(
        &mut self,
        region: Region,
        mut children: Children<'_, '_>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layout,
    ) {
        frontend.apply_brush_to_region(&attributes, region);

        let s = region.size();

        if let Some(child) = children.next() {
            child.paint(frontend, layout);
        }
    }

    fn describe(&self) -> &str {
        "border"
    }
}

pub fn border<'bp>(attr: &Attributes<'bp>) -> Box<dyn Widget<'bp> + 'bp> {
    Box::new(Border::new(attr))
}
