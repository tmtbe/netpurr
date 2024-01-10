use std::collections::BTreeMap;

use eframe::emath::pos2;
use egui::ahash::HashSet;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::{CursorRange, TextEditState};
use egui::{Context, Id, Pos2, Response, RichText, TextBuffer, TextEdit, Ui, Widget};
use serde::{Deserialize, Serialize};

use crate::data::environment::EnvironmentItemValue;
use crate::panels::VERTICAL_GAP;
use crate::utils;
use crate::utils::{popup_widget, replace_variable};
use crate::widgets::highlight::highlight_template;

pub struct HighlightTemplate<'t> {
    enable: bool,
    all_space: bool,
    size: f32,
    envs: BTreeMap<String, EnvironmentItemValue>,
    content: &'t mut dyn TextBuffer,
    popup_id: String,
    multiline: bool,
    filter: HashSet<String>,
}

impl HighlightTemplate<'_> {
    fn find_prompt(&self, input_string: String) -> Option<(String)> {
        if let Some(start_index) = input_string.rfind("{{") {
            if let Some(_) = input_string[start_index + 2..].find("}}") {
                None
            } else {
                let result = &input_string[(start_index + 2)..];
                Some(result.to_string())
            }
        } else {
            None
        }
    }

    fn popup(self, ui: &mut Ui, suggest_position: Pos2, response: Response) -> Option<Response> {
        let popup_id = ui.make_persistent_id(self.popup_id.clone());
        let mut popup_open = false;
        ui.memory_mut(|mem| {
            if mem.is_popup_open(popup_id) {
                popup_open = true
            }
        });
        if !popup_open && !response.has_focus() {
            return Some(response.clone());
        }
        HTSState::load(ui.ctx(), response.id).map(|hts_state| {
            hts_state.cursor_range.map(|c| {
                let len = self.content.as_str().len();
                if len <= 1 || c.primary.ccursor.index > len {
                    return;
                }
                let before_cursor_text: &String = &self
                    .content
                    .as_str()
                    .chars()
                    .take(c.primary.ccursor.index)
                    .collect();
                let prompt_option = self.find_prompt(before_cursor_text.to_string());
                if prompt_option.is_some() && self.envs.clone().len() > 0 {
                    let prompt = prompt_option.unwrap_or_default();
                    let mut hovered_label_key = None;
                    ui.memory_mut(|mem| mem.open_popup(popup_id));
                    popup_widget(
                        ui,
                        popup_id,
                        &response,
                        pos2(
                            suggest_position.x
                                + (c.primary.rcursor.column as f32) * (self.size / 2.0 + 1.0),
                            suggest_position.y
                                + (c.primary.rcursor.row as f32) * (self.size / 2.0 + 1.0)
                                + 16.0,
                        ),
                        |ui| {
                            ui.horizontal(|ui| {
                                egui::ScrollArea::vertical()
                                    .max_width(150.0)
                                    .max_height(400.0)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            for (key, _) in self.envs.clone().iter() {
                                                if !key.starts_with(prompt.as_str()) {
                                                    continue;
                                                }
                                                let label = utils::select_label(
                                                    ui,
                                                    RichText::new(key.as_str()).strong(),
                                                );
                                                if label.hovered() {
                                                    hovered_label_key = Some(key.clone());
                                                }
                                                if label.clicked() {
                                                    self.content.insert_text(
                                                        (key[prompt.len()..].to_string() + "}}")
                                                            .as_str(),
                                                        c.primary.ccursor.index,
                                                    );
                                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                                    let mut tes =
                                                        TextEditState::load(ui.ctx(), response.id)
                                                            .unwrap();
                                                    tes.set_ccursor_range(Some(CCursorRange {
                                                        primary: CCursor {
                                                            index: self.content.as_str().len(),
                                                            prefer_next_row: false,
                                                        },
                                                        secondary: CCursor {
                                                            index: self.content.as_str().len(),
                                                            prefer_next_row: false,
                                                        },
                                                    }));
                                                    tes.store(ui.ctx(), response.id);
                                                    response.request_focus();
                                                }
                                            }
                                        });
                                    });
                                hovered_label_key.clone().map(|key| {
                                    ui.separator();
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.strong("VALUE");
                                            ui.label(self.envs.get(&key).unwrap().value.clone())
                                        });
                                        ui.add_space(VERTICAL_GAP);
                                        ui.horizontal(|ui| {
                                            ui.strong("TYPE");
                                            ui.label(
                                                self.envs.get(&key).unwrap().value_type.to_string(),
                                            )
                                        });
                                        ui.add_space(VERTICAL_GAP);
                                        ui.horizontal(|ui| {
                                            ui.strong("SCOPE");
                                            ui.label(self.envs.get(&key).unwrap().scope.clone())
                                        });
                                    });
                                });
                            });
                        },
                    );
                } else {
                    ui.memory_mut(|mem| {
                        if mem.is_popup_open(popup_id) {
                            mem.close_popup()
                        }
                    });
                }
            });
        });
        None
    }
}

impl Widget for HighlightTemplate<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let layout_job = highlight_template(string, self.size, ui, self.envs.clone());
            ui.fonts(|f| f.layout_job(layout_job))
        };
        let filtered_string: String = self
            .content
            .as_str()
            .chars()
            .filter(|c| !self.filter.contains(c.to_string().as_str()))
            .collect();
        self.content.replace(filtered_string.as_str());
        let mut text_edit = TextEdit::singleline(self.content).layouter(&mut layouter);
        if self.multiline {
            text_edit = TextEdit::multiline(self.content).layouter(&mut layouter);
        }
        if self.all_space {
            text_edit = text_edit.desired_width(f32::INFINITY);
        }
        if !self.enable {
            ui.set_enabled(false);
        }
        let mut output = text_edit.show(ui);
        ui.set_enabled(true);
        let mut response = output.response.clone();
        let text = replace_variable(self.content.as_str().to_string(), self.envs.clone());
        if response.hovered() && text.len() > 0 && text != self.content.as_str() {
            response = response.on_hover_text(text);
        }
        output.cursor_range.map(|c| {
            let hts_state = HTSState {
                cursor_range: Some(c.clone()),
            };
            hts_state.store(ui.ctx(), response.id);
        });
        if let Some(value) = self.popup(ui, output.text_draw_pos.clone(), response.clone()) {
            return value;
        }
        response
    }
}

pub struct HighlightTemplateSinglelineBuilder {
    enable: bool,
    all_space: bool,
    size: f32,
    multiline: bool,
    filter: HashSet<String>,
    envs: BTreeMap<String, EnvironmentItemValue>,
}

impl Default for HighlightTemplateSinglelineBuilder {
    fn default() -> Self {
        HighlightTemplateSinglelineBuilder {
            enable: true,
            all_space: true,
            size: 12.0,
            multiline: false,
            filter: Default::default(),
            envs: BTreeMap::default(),
        }
    }
}

impl HighlightTemplateSinglelineBuilder {
    pub fn enable(&mut self, enable: bool) -> &mut HighlightTemplateSinglelineBuilder {
        self.enable = enable;
        self
    }
    pub fn all_space(&mut self, all_space: bool) -> &mut HighlightTemplateSinglelineBuilder {
        self.all_space = all_space;
        self
    }
    pub fn size(&mut self, size: f32) -> &mut HighlightTemplateSinglelineBuilder {
        self.size = size;
        self
    }
    pub fn envs(
        &mut self,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> &mut HighlightTemplateSinglelineBuilder {
        self.envs = envs;
        self
    }

    pub fn multiline(&mut self) -> &mut HighlightTemplateSinglelineBuilder {
        self.multiline = true;
        self
    }

    pub fn filter(&mut self, filter: HashSet<String>) -> &mut HighlightTemplateSinglelineBuilder {
        self.filter = filter;
        self
    }
    pub fn build<'t>(&'t self, id: String, content: &'t mut dyn TextBuffer) -> HighlightTemplate {
        HighlightTemplate {
            enable: self.enable,
            all_space: self.all_space,
            size: self.size,
            envs: self.envs.clone(),
            content,
            popup_id: id,
            multiline: self.multiline,
            filter: self.filter.clone(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct HTSState {
    cursor_range: Option<CursorRange>,
}

impl HTSState {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }
}
