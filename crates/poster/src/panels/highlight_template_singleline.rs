use std::collections::BTreeMap;

use eframe::emath::pos2;
use egui::text_edit::CursorRange;
use egui::{TextBuffer, TextEdit, Ui};
use uuid::Uuid;

use crate::utils::{popup_widget, replace_variable};

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
