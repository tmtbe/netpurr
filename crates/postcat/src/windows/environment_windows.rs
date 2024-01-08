use egui::{Align, Button, Checkbox, Layout, ScrollArea, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBuilder};

use crate::data::config_data::ConfigData;
use crate::data::environment::{EnvironmentConfig, EnvironmentItem, ENVIRONMENT_GLOBALS};
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::{HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct EnvironmentWindows {
    environment_windows_open: bool,
    select_env: Option<String>,
    select_env_config: EnvironmentConfig,
    select_env_name: String,
    new_select_env_item: EnvironmentItem,
}

impl Window for EnvironmentWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("MANAGE ENVIRONMENTS".to_string())
            .modal(true)
            .default_width(500.0)
            .default_height(300.0)
            .collapsible(false)
            .resizable(true)
    }

    fn set_open(&mut self, open: bool) {
        self.environment_windows_open = open
    }

    fn get_open(&self) -> bool {
        self.environment_windows_open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        _: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        if self.select_env.is_none() {
            self.env_list(workspace_data, ui);
        } else {
            self.select_modify(ui);
        }
        self.env_bottom(workspace_data, ui);
    }
}

impl EnvironmentWindows {
    fn env_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.label("An environment is a set of variables that allow you to switch the context of your requests. Environments can be shared between multiple workspaces.");
        ui.add_space(VERTICAL_GAP * 2.0);
        ScrollArea::vertical().show(ui, |ui| {
            for (name, e) in workspace_data.get_env_configs().iter() {
                if name == ENVIRONMENT_GLOBALS {
                    continue;
                }
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP * 3.0);
                    utils::left_right_panel(
                        ui,
                        "env_".to_string() + name.as_str(),
                        |ui| {
                            if ui.hyperlink(name).clicked() {
                                self.select_env = Some(name.clone());
                                self.select_env_config = e.clone();
                                self.select_env_name = name.clone();
                            }
                        },
                        |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("📋").clicked() {
                                    workspace_data.add_env(name.to_string() + " Copy", e.clone());
                                };
                                ui.button("⬇");
                                if ui.button("🗑").clicked() {
                                    workspace_data.remove_env(name.to_string());
                                }
                            });
                        },
                    );
                });
            }
        });
    }

    fn select_modify(&mut self, ui: &mut Ui) {
        if self.select_env_name == ENVIRONMENT_GLOBALS {
            ui.label("Global variables for a workspace are a set of variables that are always available within the scope of that workspace. They can be viewed and edited by anyone in that workspace.");
        } else {
            ui.strong("Environment Name");
            TextEdit::singleline(&mut self.select_env_name)
                .desired_width(f32::INFINITY)
                .ui(ui);
        }
        ui.add_space(VERTICAL_GAP * 2.0);
        let mut delete_index = None;
        ui.push_id("environment_table", |ui| {
            let table = TableBuilder::new(ui)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto())
                .column(Column::exact(20.0))
                .column(Column::initial(200.0).range(40.0..=300.0))
                .column(Column::remainder())
                .max_scroll_height(400.0);
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("VARIABLE");
                    });
                    header.col(|ui| {
                        ui.strong("VALUE");
                    });
                })
                .body(|mut body| {
                    for (index, item) in self.select_env_config.items.iter_mut().enumerate() {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut item.enable, "");
                            });
                            row.col(|ui| {
                                if ui.button("x").clicked() {
                                    delete_index = Some(index)
                                }
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut item.key);
                            });
                            row.col(|ui| {
                                TextEdit::singleline(&mut item.value)
                                    .desired_width(f32::INFINITY)
                                    .ui(ui);
                            });
                        });
                    }
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.add_enabled(
                                false,
                                Checkbox::new(&mut self.new_select_env_item.enable, ""),
                            );
                        });
                        row.col(|ui| {
                            ui.add_enabled(false, Button::new("x"));
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut self.new_select_env_item.key);
                        });
                        row.col(|ui| {
                            TextEdit::singleline(&mut self.new_select_env_item.value)
                                .desired_width(f32::INFINITY)
                                .ui(ui);
                        });
                    });
                });
        });
        if delete_index.is_some() {
            self.select_env_config.items.remove(delete_index.unwrap());
        }
        if self.new_select_env_item.key != "" || self.new_select_env_item.value != "" {
            self.new_select_env_item.enable = true;
            self.select_env_config
                .items
                .push(self.new_select_env_item.clone());
            self.new_select_env_item.key = "".to_string();
            self.new_select_env_item.value = "".to_string();
            self.new_select_env_item.enable = false;
        }
    }

    fn env_bottom(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("environment_bottom_panel")
            .resizable(false)
            .min_height(0.0)
            .show_inside(ui, |ui| {
                ui.add_space(VERTICAL_GAP);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.select_env.is_none() {
                        if ui.button("Add").clicked() {
                            self.select_env = Some("".to_string());
                            self.select_env_config = EnvironmentConfig::default();
                            self.select_env_name = "".to_string();
                        }
                        ui.button("Import");
                        if ui.button("Globals").clicked() {
                            let data = workspace_data.get_env(ENVIRONMENT_GLOBALS.to_string());
                            self.select_env = Some(ENVIRONMENT_GLOBALS.to_string());
                            self.select_env_config = data.unwrap_or(EnvironmentConfig::default());
                            self.select_env_name = ENVIRONMENT_GLOBALS.to_string();
                        };
                    } else {
                        if ui.button("Update").clicked() {
                            if self.select_env_name != "" {
                                workspace_data.remove_env(self.select_env.clone().unwrap());
                                workspace_data.add_env(
                                    self.select_env_name.clone(),
                                    self.select_env_config.clone(),
                                );
                                self.select_env = None;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.select_env = None
                        }
                    }
                });
            });
    }
}
