use std::collections::BTreeMap;

use eframe::emath::{Align, Pos2};
use eframe::epaint::text::LayoutJob;
use egui::text_edit::CursorRange;
use egui::{
    pos2, Area, FontSelection, Frame, Id, InnerResponse, Key, Layout, Order, Response, RichText,
    Style, TextBuffer, TextEdit, Ui, Widget, WidgetText,
};
use regex::Regex;
use uuid::Uuid;

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
    lb.break_on_newline = false;
    lb.wrap.max_width = f32::INFINITY;
    lb.wrap.max_rows = 2;
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

pub struct HighlightTemplateSingleline {
    enable: bool,
    all_space: bool,
    size: f32,
    cursor_range: Option<CursorRange>,
    popup_id: String,
}

impl Default for HighlightTemplateSingleline {
    fn default() -> Self {
        HighlightTemplateSingleline {
            enable: false,
            all_space: false,
            size: 12.0,
            cursor_range: None,
            popup_id: Uuid::new_v4().to_string(),
        }
    }
}

impl HighlightTemplateSingleline {
    pub fn new(enable: bool, all_space: bool, size: f32) -> Self {
        HighlightTemplateSingleline {
            enable,
            all_space,
            size,
            cursor_range: None,
            popup_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn set(
        &mut self,
        id: String,
        enable: bool,
        all_space: bool,
        size: f32,
    ) -> &mut HighlightTemplateSingleline {
        self.enable = enable;
        self.all_space = all_space;
        self.size = size;
        if self.popup_id != id {
            self.cursor_range = None;
            self.popup_id = id;
        }
        return self;
    }
    pub fn show(
        &mut self,
        ui: &mut Ui,
        content: &mut dyn TextBuffer,
        envs: BTreeMap<String, String>,
    ) {
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let layout_job =
                crate::widgets::highlight::highlight_template(string, self.size, ui, envs.clone());
            ui.fonts(|f| f.layout_job(layout_job))
        };
        let mut text_edit = TextEdit::singleline(content).layouter(&mut layouter);
        if self.all_space {
            text_edit = text_edit.desired_width(f32::INFINITY);
        }
        if !self.enable {
            ui.set_enabled(false);
        }
        let mut output = text_edit.show(ui);
        ui.set_enabled(true);
        let mut response = output.response;
        let text = replace_variable(content.as_str().to_string(), envs.clone());
        if response.hovered() && text.len() > 0 && text != content.as_str() {
            response = response.on_hover_text(text);
        }
        output.cursor_range.map(|c| {
            self.cursor_range = Some(c.clone());
        });
        let popup_id = ui.make_persistent_id(self.popup_id.clone());
        self.cursor_range.map(|c| {
            if content.as_str().len() <= 1 {
                return;
            }
            let before_cursor_text = &content.as_str()[0..c.primary.ccursor.index];
            if before_cursor_text.ends_with("{{") && envs.clone().len() > 0 {
                ui.memory_mut(|mem| mem.open_popup(popup_id));
            }
            popup_widget(
                ui,
                popup_id,
                &response,
                pos2(
                    output.text_draw_pos.x
                        + (c.primary.rcursor.column as f32) * (self.size / 2.0 + 1.0),
                    output.text_draw_pos.y
                        + (c.primary.rcursor.row as f32) * (self.size / 2.0 + 1.0)
                        + 16.0,
                ),
                |ui| {
                    ui.vertical(|ui| {
                        for (key, _) in envs.clone().iter() {
                            if ui
                                .selectable_label(false, "$".to_string() + key.as_str())
                                .clicked()
                            {
                                content.insert_text(
                                    (key.to_string() + "}}").as_str(),
                                    c.primary.ccursor.index,
                                );
                                self.cursor_range = None;
                                ui.memory_mut(|mem| mem.close_popup());
                                response.request_focus();
                            }
                        }
                    });
                },
            );
        });
    }
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

pub fn replace_variable(content: String, envs: BTreeMap<String, String>) -> String {
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
