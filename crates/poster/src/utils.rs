use eframe::emath::Align;
use eframe::epaint::text::LayoutJob;
use egui::ahash::HashMap;
use egui::{
    FontSelection, Id, InnerResponse, RichText, Style, TextBuffer, TextEdit, Ui, Widget, WidgetText,
};
use regex::Regex;

use crate::data::Request;
use crate::panels::HORIZONTAL_GAP;
use crate::widgets::selectable_value_with_close_button::{
    ExResponse, SelectableLabelWithCloseButton,
};

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

pub fn selectable_label_with_close_button<Value: PartialEq>(
    ui: &mut Ui,
    current_value: &mut Value,
    selected_value: Value,
    text: impl Into<WidgetText>,
) -> ExResponse {
    let mut ex_response =
        _selectable_label_with_close_button(ui, *current_value == selected_value, text);
    if ex_response.clicked() && *current_value != selected_value {
        *current_value = selected_value;
        ex_response.response().mark_changed();
    }
    ex_response
}

#[must_use = "You should check if the user clicked this with `if ui.selectable_label(…).clicked() { … } "]
fn _selectable_label_with_close_button(
    ui: &mut Ui,
    checked: bool,
    text: impl Into<WidgetText>,
) -> ExResponse {
    SelectableLabelWithCloseButton::new(checked, text).ui(ui)
}

pub fn build_with_count_ui_header(name: String, count: usize, ui: &Ui) -> LayoutJob {
    let mut lb = LayoutJob::default();
    let style = Style::default();
    RichText::new(name + " ")
        .color(ui.visuals().text_color())
        .strong()
        .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
    if count > 0 {
        RichText::new("(".to_string() + count.to_string().as_str() + ")")
            .color(ui.visuals().warn_fg_color)
            .strong()
            .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
    }
    lb
}

pub fn left_right_panel(
    ui: &mut Ui,
    left_id: impl Into<Id>,
    left: impl FnOnce(&mut Ui),
    right_id: impl Into<Id>,
    right: impl FnOnce(&mut Ui),
) -> InnerResponse<()> {
    ui.horizontal(|ui| {
        egui::SidePanel::right(right_id)
            .resizable(true)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                right(ui);
            });
        egui::SidePanel::left(left_id)
            .resizable(true)
            .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
            .show_inside(ui, |ui| {
                left(ui);
            });
    })
}

pub fn highlight_template_singleline(
    ui: &mut Ui,
    content: &mut dyn TextBuffer,
    envs: HashMap<String, String>,
) {
    let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
        let layout_job = crate::widgets::highlight::highlight_template(string, ui, envs.clone());
        ui.fonts(|f| f.layout_job(layout_job))
    };
    let response = TextEdit::singleline(content).layouter(&mut layouter).ui(ui);
    let text = replace_variable(content.as_str().to_string(), envs);
    if response.hovered() && text.len() > 0 && text != content.as_str() {
        response.on_hover_text(text);
    }
}

pub fn replace_variable(content: String, envs: HashMap<String, String>) -> String {
    let re = Regex::new(r"\{\{.*?}}").unwrap();
    let mut result = content.clone();
    loop {
        let temp = result.clone();
        let find = re.find_iter(temp.as_str()).next();
        if find.is_some() {
            let key = find
                .unwrap()
                .as_str()
                .trim_start_matches("{{")
                .trim_end_matches("}}");
            let v = envs.get(key);
            if v.is_some() {
                result.replace_range(find.unwrap().range(), v.unwrap())
            } else {
                result.replace_range(find.unwrap().range(), "{UNKNOWN}")
            }
        } else {
            break;
        }
    }
    result
}
