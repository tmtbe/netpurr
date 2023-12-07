use eframe::emath::Align;
use egui::{Layout, Response, SelectableLabel, Ui, Widget, WidgetText};

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct SelectableLabelWithCloseButton {
    label: SelectableLabel,
}

impl SelectableLabelWithCloseButton {
    pub fn new(selected: bool, text: impl Into<WidgetText>) -> Self {
        Self {
            label: SelectableLabel::new(selected, text),
        }
    }
}

pub struct ExResponse {
    response: Response,
    closed: bool,
    clicked: bool,
}

impl ExResponse {
    pub fn response(&mut self) -> &mut Response {
        &mut self.response
    }
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub fn clicked(&self) -> bool {
        self.clicked
    }
}

impl SelectableLabelWithCloseButton {
    pub fn ui(self, ui: &mut Ui) -> ExResponse {
        let mut closed = false;
        let mut clicked = false;
        ExResponse {
            response: ui
                .with_layout(Layout::left_to_right(Align::Center), |ui| {
                    clicked = self.label.ui(ui).clicked();
                    closed = ui.button("x").clicked()
                })
                .response,
            clicked,
            closed,
        }
    }
}
//
// fn with_layout_dyn<'c, R>(
//     ui: &mut Ui,
//     layout: Layout,
//     add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
// ) -> InnerResponse<R> {
//     let mut child_ui = ui.child_ui(ui.available_rect_before_wrap(), layout);
//     let inner = add_contents(&mut child_ui);
//     let rect = child_ui.min_rect();
//     let item_spacing = ui.spacing().item_spacing;
//     ui.placer.advance_after_rects(rect, rect, item_spacing);
//
//     InnerResponse::new(inner, self.interact(rect, child_ui.id, Sense::hover()))
// }
