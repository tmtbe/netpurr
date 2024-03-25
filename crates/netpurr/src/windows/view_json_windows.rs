use egui::{Align, Button, Layout, Ui};
use egui_json_tree::{DefaultExpand, JsonTree};
use serde_json::Value;

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use netpurr_core::data::workspace_data::WorkspaceData;

#[derive(Default)]
pub struct ViewJsonWindows {
    open: bool,
    json: String,
    search_input: String,
    id: String,
}

impl ViewJsonWindows {
    pub fn with_json(mut self, json: String, id: String) -> Self {
        self.json = json;
        self.id = id;
        self
    }
}

impl Window for ViewJsonWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new_with_id("VIEW JSON TREE", self.id.clone())
            .max_height(500.0)
            .max_width(500.0)
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn get_open(&self) -> bool {
        self.open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        match serde_json::from_str::<Value>(self.json.as_str()) {
            Ok(value) => {
                ui.label("Search:");
                let (text_edit_response, clear_button_response) = ui
                    .horizontal(|ui| {
                        let text_edit_response = ui.text_edit_singleline(&mut self.search_input);
                        let clear_button_response = ui.button("Clear");
                        (text_edit_response, clear_button_response)
                    })
                    .inner;
                egui::ScrollArea::vertical()
                    .max_height(600.0)
                    .show(ui, |ui| {
                        let response = JsonTree::new(
                            "json_tree_".to_string() + self.window_setting().id(),
                            &value,
                        )
                        .default_expand(DefaultExpand::SearchResults(&self.search_input))
                        .response_callback(|response, pointer| {
                            response.context_menu(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                                    ui.set_width(150.0);
                                    if !pointer.is_empty()
                                        && ui
                                            .add(Button::new("Copy property path").frame(false))
                                            .clicked()
                                    {
                                        operation.add_success_toast("Copy path success.");
                                        ui.output_mut(|o| o.copied_text = pointer.clone());
                                        ui.close_menu();
                                    }

                                    if ui.add(Button::new("Copy contents").frame(false)).clicked() {
                                        if let Some(val) = value.pointer(pointer) {
                                            if let Ok(pretty_str) =
                                                serde_json::to_string_pretty(val)
                                            {
                                                ui.output_mut(|o| o.copied_text = pretty_str);
                                                operation
                                                    .add_success_toast("Copy contents success.");
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                });
                            });
                        })
                        .show(ui);

                        if text_edit_response.changed() {
                            response.reset_expanded(ui);
                        }

                        if clear_button_response.clicked() {
                            self.search_input.clear();
                            response.reset_expanded(ui);
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Reset expanded").clicked() {
                                response.reset_expanded(ui);
                            }
                            if ui.button("Copy json").clicked() {
                                ui.output_mut(|o| o.copied_text = self.json.clone());
                                operation.add_success_toast("Copy json success.");
                            }
                        });
                    });
            }
            Err(_) => {
                ui.label("Error Json Format");
            }
        };
    }
}
