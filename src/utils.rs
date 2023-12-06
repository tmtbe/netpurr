use eframe::emath::Align;
use eframe::epaint::text::LayoutJob;
use egui::{FontSelection, RichText, Style, Ui};

use crate::data::Request;

pub fn build_rest_ui_header(request: Request, ui: &Ui) -> LayoutJob {
    let mut lb = LayoutJob::default();
    let style = Style::default();
    if request.base_url != "" {
        RichText::new(request.method.to_string() + " ")
            .color(ui.visuals().warn_fg_color)
            .strong()
            .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
        RichText::new(request.base_url.to_string())
            .color(ui.visuals().text_color())
            .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
    } else {
        RichText::new("Untitled Request")
            .strong()
            .color(ui.visuals().text_color())
            .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
    }
    lb
}
