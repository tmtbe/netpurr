use std::cmp::min;
use std::collections::HashSet;
use chrono::format;

use eframe::emath::{Align, Pos2};
use eframe::epaint::text::LayoutJob;
use egui::{
    Area, CollapsingHeader, CollapsingResponse, Color32, FontSelection, Frame, Id, InnerResponse,
    Key, Layout, Order, Response, RichText, Style, TextBuffer, Ui, WidgetText,
};
use egui::text::TextWrapping;

use netpurr_core::data::record::Record;

use crate::panels::HORIZONTAL_GAP;

pub mod openapi_help;

pub fn build_rest_ui_header(record: Record, max_char: Option<usize>, ui: &Ui) -> LayoutJob {
    let mut lb = LayoutJob {
        text: Default::default(),
        sections: Default::default(),
        wrap: TextWrapping {
            max_width: 50.0,
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Some('…'),
        },
        first_row_min_height: 0.0,
        break_on_newline: false,
        halign: Align::LEFT,
        justify: false,
    };
    let style = Style::default();
    RichText::new(format!("{} ", egui_phosphor::regular::FILE_TEXT)).append_to(&mut lb, &style, FontSelection::Default, Align::Center);
    if record.base_url() != "" {
        RichText::new(record.method() + " ")
            .color(ui.visuals().warn_fg_color)
            .strong()
            .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
        let mut new_name = "".to_string();
        if record.name() != "" {
            new_name = record.name();
        } else {
            new_name = record.base_url();
        }
        match max_char {
            None => {}
            Some(size) => {
                if new_name.len() > size {
                    let len = min(new_name.chars().count() - 1, size);
                    new_name = new_name.chars().take(len).collect::<String>() + "...";
                }
            }
        }
        RichText::new(new_name)
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

pub enum HighlightValue {
    None,
    Has,
    Usize(usize),
    String(String, Color32),
}

pub fn build_with_count_ui_header(
    name: String,
    highlight_value: HighlightValue,
    ui: &Ui,
) -> LayoutJob {
    let mut lb = LayoutJob::default();
    let mut color = Color32::GREEN;
    let style = Style::default();
    RichText::new(name + " ")
        .color(ui.visuals().text_color())
        .strong()
        .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
    match highlight_value {
        HighlightValue::Has => {
            RichText::new("●").color(color.clone()).strong().append_to(
                &mut lb,
                &style,
                FontSelection::Default,
                Align::Center,
            );
        }
        HighlightValue::Usize(value) => {
            RichText::new("(".to_string() + value.to_string().as_str() + ")")
                .color(color.clone())
                .strong()
                .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
        }
        HighlightValue::String(value, self_color) => {
            RichText::new("(".to_string() + value.as_str() + ")")
                .color(self_color)
                .strong()
                .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
        }
        HighlightValue::None => {}
    }
    lb
}

pub fn left_right_panel(
    ui: &mut Ui,
    id: String,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) -> InnerResponse<()> {
    let left_id = id.clone() + "_left";
    let right_id = id.clone() + "_right";
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

pub fn popup_widget<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    suggested_position: Pos2,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let inner = Area::new(popup_id)
            .order(Order::Foreground)
            .constrain(true)
            .fixed_pos(suggested_position)
            .show(ui.ctx(), |ui| {
                let frame = Frame::popup(ui.style());
                frame
                    .show(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| add_contents(ui))
                            .inner
                    })
                    .inner
            })
            .inner;

        if ui.input(|i| i.key_pressed(Key::Escape)) || widget_response.clicked_elsewhere() {
            ui.memory_mut(|mem| mem.close_popup());
        }
        Some(inner)
    } else {
        None
    }
}

pub fn select_label(ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
    ui.with_layout(
        Layout::top_down(Align::LEFT).with_cross_justify(true),
        |ui| ui.selectable_label(false, text),
    )
    .inner
}

pub fn select_value<Value: PartialEq>(
    ui: &mut Ui,
    current_value: &mut Value,
    selected_value: Value,
    text: impl Into<WidgetText>,
) -> Response {
    ui.with_layout(
        Layout::top_down(Align::LEFT).with_cross_justify(true),
        |ui| ui.selectable_value(current_value, selected_value, text),
    )
    .inner
}

pub fn text_edit_singleline_justify<S: TextBuffer>(ui: &mut Ui, text: &mut S) -> Response {
    ui.with_layout(
        Layout::top_down(Align::LEFT).with_cross_justify(true),
        |ui| ui.text_edit_singleline(text),
    )
    .inner
}

pub fn text_edit_singleline_filter_justify<S: TextBuffer>(ui: &mut Ui, text: &mut S) -> Response {
    text.replace_with(
        text.as_str()
            .replace("/", "_")
            .as_str()
            .replace(" ", "_")
            .as_str(),
    );
    let filtered_string: String = text
        .as_str()
        .chars()
        .filter(|&c| c.is_ascii_alphabetic() || c.is_alphabetic() || c.is_numeric() || c == '_')
        .collect();
    text.replace_with(filtered_string.as_str());
    ui.with_layout(
        Layout::top_down(Align::LEFT).with_cross_justify(true),
        |ui| ui.text_edit_singleline(text),
    )
    .inner
}

pub fn text_edit_singleline_filter<S: TextBuffer>(ui: &mut Ui, text: &mut S) -> Response {
    text.replace_with(
        text.as_str()
            .replace("/", "_")
            .as_str()
            .replace(" ", "_")
            .as_str(),
    );
    let filtered_string: String = text
        .as_str()
        .chars()
        .filter(|&c| c.is_ascii_alphabetic() || c.is_alphabetic() || c.is_numeric() || c == '_')
        .collect();
    text.replace_with(filtered_string.as_str());
    ui.text_edit_singleline(text)
}

pub fn text_edit_multiline_justify<S: TextBuffer>(ui: &mut Ui, text: &mut S) -> Response {
    ui.with_layout(
        Layout::top_down(Align::LEFT).with_cross_justify(true),
        |ui| ui.text_edit_multiline(text),
    )
    .inner
}

pub fn build_copy_name(mut name: String, names: HashSet<String>) -> String {
    name = name
        .splitn(2, "Copy")
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    let mut index = 2;
    let mut new_name = name.clone();
    while (names.contains(new_name.as_str())) {
        new_name = format!("{} Copy {}", name.clone(), index);
        index += 1;
    }
    return new_name;
}

pub fn selectable_check<Value: PartialEq>(
    ui: &mut Ui,
    current_value: &mut Value,
    selected_value: Value,
    text: impl Into<WidgetText>,
) -> Response {
    let mut response = ui.checkbox(&mut (*current_value == selected_value), text);
    if response.clicked() && *current_value != selected_value {
        *current_value = selected_value;
        response.mark_changed();
    }
    response
}

pub fn add_right_space(ui: &mut Ui, space: f32) {
    let space = ui.available_width() - space;
    if space > 0.0 {
        ui.add_space(space)
    }
}

pub fn add_left_space(ui: &mut Ui, space: f32) {
    let space = space;
    if space > 0.0 {
        ui.add_space(space)
    }
}

pub fn open_collapsing<R>(
    ui: &mut Ui,
    heading: impl Into<WidgetText>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> CollapsingResponse<R> {
    CollapsingHeader::new(heading)
        .default_open(true)
        .show(ui, add_contents)
}
