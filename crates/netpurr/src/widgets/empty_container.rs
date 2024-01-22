use eframe::emath::{pos2, vec2, Align2, Rect, Vec2};
use eframe::epaint::Stroke;
use egui::{Id, Response, Shape, Ui};

/// A region that can be resized by dragging the bottom right corner.
#[derive(Clone, Copy, Debug)]
#[must_use = "You should call .show()"]
pub struct EmptyContainer {
    id: Option<Id>,
    id_source: Option<Id>,
    default_size: Vec2,
    with_stroke: bool,
}

impl Default for EmptyContainer {
    fn default() -> Self {
        Self {
            id: None,
            id_source: None,
            default_size: vec2(320.0, 128.0),
            with_stroke: true,
        }
    }
}

impl EmptyContainer {
    /// Assign an explicit and globally unique id.
    #[inline]
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// A source for the unique [`Id`], e.g. `.id_source("second_resize_area")` or `.id_source(loop_index)`.
    #[inline]
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Preferred / suggested width. Actual width will depend on contents.
    ///
    /// Examples:
    /// * if the contents is text, this will decide where we break long lines.
    /// * if the contents is a canvas, this decides the width of it,
    /// * if the contents is some buttons, this is ignored and we will auto-size.
    #[inline]
    pub fn default_width(mut self, width: f32) -> Self {
        self.default_size.x = width;
        self
    }

    /// Preferred / suggested height. Actual height will depend on contents.
    ///
    /// Examples:
    /// * if the contents is a [`ScrollArea`] then this decides the maximum size.
    /// * if the contents is a canvas, this decides the height of it,
    /// * if the contents is text and buttons, then the `default_height` is ignored
    ///   and the height is picked automatically..
    #[inline]
    pub fn default_height(mut self, height: f32) -> Self {
        self.default_size.y = height;
        self
    }

    #[inline]
    pub fn default_size(mut self, default_size: impl Into<Vec2>) -> Self {
        self.default_size = default_size.into();
        self
    }

    #[inline]
    pub fn with_stroke(mut self, with_stroke: bool) -> Self {
        self.with_stroke = with_stroke;
        self
    }
}

struct Prepared {
    id: Id,
    content_ui: Ui,
}

impl EmptyContainer {
    fn begin(&mut self, ui: &mut Ui) -> Prepared {
        let position = ui.available_rect_before_wrap().min;
        let id = self.id.unwrap_or_else(|| {
            let id_source = self.id_source.unwrap_or_else(|| Id::new("resize"));
            ui.make_persistent_id(id_source)
        });
        let inner_rect = Rect::from_min_size(position, self.default_size);

        let mut content_clip_rect = inner_rect.expand(ui.visuals().clip_rect_margin);
        content_clip_rect = content_clip_rect.intersect(ui.clip_rect()); // Respect parent region

        let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
        content_ui.set_clip_rect(content_clip_rect);

        Prepared { id, content_ui }
    }

    pub fn show<R>(mut self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let mut prepared = self.begin(ui);
        let ret = add_contents(&mut prepared.content_ui);
        self.end(ui, prepared);
        ret
    }

    fn end(self, ui: &mut Ui, prepared: Prepared) {
        let Prepared { id, content_ui } = prepared;

        ui.advance_cursor_after_rect(Rect::from_min_size(
            content_ui.min_rect().min,
            self.default_size,
        ));

        // ------------------------------

        if self.with_stroke {
            let rect = Rect::from_min_size(content_ui.min_rect().left_top(), self.default_size);
            let rect = rect.expand(2.0); // breathing room for content
            ui.painter().add(Shape::rect_stroke(
                rect,
                3.0,
                ui.visuals().widgets.noninteractive.bg_stroke,
            ));
        }
    }
}

pub fn paint_resize_corner(ui: &Ui, response: &Response) {
    let stroke = ui.style().interact(response).fg_stroke;
    paint_resize_corner_with_style(ui, &response.rect, stroke, Align2::RIGHT_BOTTOM);
}

pub fn paint_resize_corner_with_style(
    ui: &Ui,
    rect: &Rect,
    stroke: impl Into<Stroke>,
    corner: Align2,
) {
    let painter = ui.painter();
    let cp = painter.round_pos_to_pixels(corner.pos_in_rect(rect));
    let mut w = 2.0;
    let stroke = stroke.into();

    while w <= rect.width() && w <= rect.height() {
        painter.line_segment(
            [
                pos2(cp.x - w * corner.x().to_sign(), cp.y),
                pos2(cp.x, cp.y - w * corner.y().to_sign()),
            ],
            stroke,
        );
        w += 4.0;
    }
}
