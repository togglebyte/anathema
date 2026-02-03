use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_runtime::widgets::{Children, LayoutSize, Layouts, Widget};
use anathema_runtime::{Attributes, Constraints, TemplateValue, WidgetAttributes};
use compact_str::CompactString;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const CORNER_COUNT: usize = 8;

mod corners {
    pub(super) const TOP_LEFT: usize = 0;
    pub(super) const TOP: usize = 1;
    pub(super) const TOP_RIGHT: usize = 2;
    pub(super) const RIGHT: usize = 3;
    pub(super) const BOTTOM_RIGHT: usize = 4;
    pub(super) const BOTTOM: usize = 5;
    pub(super) const BOTTOM_LEFT: usize = 6;
    pub(super) const LEFT: usize = 7;
}

const DEFAULT_SLIM_EDGES: CompactString = CompactString::const_new("┌─┐│┘─└│");
const DEFAULT_THICK_EDGES: CompactString = CompactString::const_new("╔═╗║╝═╚║");
const DEFAULT_ROUND_EDGES: CompactString = CompactString::const_new("╭─╮│╯─╰│");

type Index = u8;

struct BorderCorners<'a> {
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
struct BorderStyle {
    kind: CompactString,
    left_width: u8,
    right_width: u8,
    corners: [Index; CORNER_COUNT],
}

impl BorderStyle {
    const fn empty() -> Self {
        Self {
            kind: DEFAULT_SLIM_EDGES,
            left_width: 1,
            right_width: 1,
            corners: [0, 3, 6, 9, 12, 15, 18, 21],
        }
    }

    fn size(&self) -> Size {
        let width = self.left_width + self.right_width;
        Size::new(width as u32, 2)
    }

    fn fix_if_expired(&mut self, kind: &str) {
        if kind == self.kind {
            return;
        }

        let graphemes = self.kind.grapheme_indices(true);
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
        &self.kind[from..to]
    }

    fn top_left(&self) -> &str {
        self.slice(corners::TOP_LEFT, corners::TOP)
    }

    fn top(&self) -> &str {
        self.slice(corners::TOP, corners::TOP_RIGHT)
    }

    fn top_right(&self) -> &str {
        self.slice(corners::TOP_RIGHT, corners::RIGHT)
    }

    fn right(&self) -> &str {
        self.slice(corners::RIGHT, corners::BOTTOM_RIGHT)
    }

    fn bottom_right(&self) -> &str {
        self.slice(corners::BOTTOM_RIGHT, corners::BOTTOM)
    }

    fn bottom(&self) -> &str {
        self.slice(corners::BOTTOM, corners::BOTTOM_LEFT)
    }

    fn bottom_left(&self) -> &str {
        self.slice(corners::BOTTOM_LEFT, corners::LEFT)
    }

    fn left(&self) -> &str {
        let from = self.corners[corners::LEFT] as usize;
        let to = self.kind.len();
        &self.kind[from..to]
    }
}

struct BorderPaint<'a> {
    region: Region,
    style: &'a BorderStyle,
    frontend: &'a mut dyn Frontend,
}

impl<'a> BorderPaint<'a> {
    fn new(region: Region, style: &'a BorderStyle, frontend: &'a mut dyn Frontend) -> Self {
        Self {
            region,
            style,
            frontend,
        }
    }

    fn horz(&mut self) {
        let width = self.region.size().width as usize;

        let top = self.style.top();
        let top_pos = self.region.from + Pos::new(self.style.top_left().width() as i32, 0);
        let top_width = width - self.style.top_right().width();
        self.frontend.repeat_text(top, top_width, top_pos);

        let bottom = self.style.bottom();
        let bottom_pos = Pos::new(self.region.from.x, self.region.to.y - 1);
        let bottom_width = width - self.style.bottom_right().width();
        self.frontend.repeat_text(bottom, bottom_width, bottom_pos);
    }

    fn vert(&mut self) {
        let left = self.style.left();
        let right = self.style.right();

        for y in self.region.from.y + 1..self.region.to.y - 1 {
            let left_x = self.region.from.x;
            let right_x = self.region.to.x - right.width() as i32;
            self.frontend.set_text(left, Pos::new(left_x, y as i32));
            self.frontend.set_text(right, Pos::new(right_x, y as i32));
        }
    }

    fn corners(mut self) {
        let top_left = self.region.from;
        self.frontend.set_text(self.style.top_left(), top_left);

        let top_right = Pos::new(
            self.region.to.x - self.style.top_right().width() as i32,
            self.region.from.y,
        );
        self.frontend.set_text(self.style.top_right(), top_right);

        let bottom_right = Pos::new(
            self.region.to.x - self.style.top_right().width() as i32,
            self.region.to.y - 1,
        );
        self.frontend.set_text(self.style.bottom_right(), bottom_right);

        let bottom_left = Pos::new(self.region.from.x, self.region.to.y - 1);
        self.frontend.set_text(self.style.bottom_left(), bottom_left);
    }

    fn paint(mut self) {
        // -----------------------------------------------------------------------------
        //   - Horz -
        // -----------------------------------------------------------------------------
        self.horz();

        // -----------------------------------------------------------------------------
        //   - Vert -
        // -----------------------------------------------------------------------------
        self.vert();

        // -----------------------------------------------------------------------------
        //   - Corners -
        // -----------------------------------------------------------------------------
        self.corners();
    }
}

#[derive(Debug, Default)]
pub struct Border {
    border: BorderStyle,
}

impl Border {
    pub(crate) fn new<'bp>(attrs: &Attributes<'bp>) -> Self {
        Self {
            border: BorderStyle::empty(),
        }
    }
}

impl<'bp> Widget<'bp> for Border {
    fn layout(
        &mut self,
        mut children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        mut constraints: Constraints,
    ) -> LayoutSize {
        crate::update_constraints(attributes, &mut constraints);
        let border_size = self.border.size();

        let inner = children
            .next()
            .map(|child| child.layout(layout, constraints - border_size))
            .map(|size| size)
            .unwrap_or(Size::ZERO);

        // if there is a fixed dimension then use that as long as it's smaller than
        // or equal to that of the max constraint
        let outer = crate::fix_size(inner + border_size, attributes, constraints);

        LayoutSize::new(inner, outer)
    }

    fn position(
        &mut self,
        mut children: Children<'_, '_>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layouts,
        mut pos: Pos,
    ) {
        let Some(child) = children.next() else { return };
        pos.x += self.border.left_width as i32;
        pos.y += 1;
        child.position(layout, pos);
    }

    fn paint(
        &mut self,
        region: Region,
        mut children: Children<'_, '_>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layout: &Layouts,
    ) {
        frontend.apply_brush_to_region(&attributes, region);

        if let Some(child) = children.next() {
            child.paint(frontend, layout);
        }

        let painter = BorderPaint::new(region, &self.border, frontend);
        painter.paint();
    }

    fn describe(&self) -> &str {
        "border"
    }
}

pub fn border<'bp>(attr: &Attributes<'bp>) -> Box<dyn Widget<'bp> + 'bp> {
    Box::new(Border::new(attr))
}
