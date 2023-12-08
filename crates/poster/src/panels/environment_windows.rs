use egui::{Align, Button, Checkbox, Layout, ScrollArea, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBuilder};

use crate::data::{AppData, EnvironmentConfig, EnvironmentItem};
use crate::panels::{DataView, HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct EnvironmentWindows {
    environment_windows_open: bool,
    select_env: Option<String>,
    select_env_config: EnvironmentConfig,
    select_env_name: String,
    new_select_env_item: EnvironmentItem,
}

impl EnvironmentWindows {
    pub(crate) fn open(&mut self) {
        self.environment_windows_open = true;
    }
}

impl DataView for EnvironmentWindows {
    type CursorType = i32;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        egui::Window::new("MANAGE ENVIRONMENTS")
            .default_open(true)
            .default_width(500.0)
            .default_height(300.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut self.environment_windows_open)
            .show(ui.ctx(), |ui| {
                if self.select_env.is_none() {
                    ui.label("An environment is a set of variables that allow you to switch the context of your requests. Environments can be shared between multiple workspaces.");
                    ui.add_space(VERTICAL_GAP * 2.0);
                    ScrollArea::vertical()
                        .show(ui, |ui| {
                            for (name, e) in app_data.environment.data.clone().iter() {
                                ui.horizontal(|ui| {
                                    ui.add_space(HORIZONTAL_GAP * 3.0);
                                    utils::left_right_panel(ui, "env_".to_string() + name + "_left", |ui| {
                                        if ui.hyperlink(name).clicked() {
                                            self.select_env = Some(name.clone());
                                            self.select_env_config = e.clone();
                                            self.select_env_name = name.clone();
                                        }
                                    }, "env_".to_string() + name + "_right", |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("📋").clicked() {
                                                app_data.environment.data.insert(name.to_string() + " Copy", e.clone());
                                            };
                                            ui.button("⬇");
                                            if ui.button("🗑").clicked() {
                                                app_data.environment.data.remove(name);
                                            }
                                        });
                                    });
                                });
                            }
                        });
                } else {
                    ui.strong("Environment Name");
                    TextEdit::singleline(&mut self.select_env_name)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
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
                            .min_scrolled_height(200.0);
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
                                        ui.add_enabled(false, Checkbox::new(&mut self.new_select_env_item.enable, ""));
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
                        self.select_env_config.items.push(self.new_select_env_item.clone());
                        self.new_select_env_item.key = "".to_string();
                        self.new_select_env_item.value = "".to_string();
                        self.new_select_env_item.enable = false;
                    }
                }
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
                                ui.button("Globals");
                            } else {
                                if ui.button("Update").clicked() {
                                    if self.select_env_name != "" {
                                        app_data.environment.data.remove(self.select_env.clone().unwrap().as_str());
                                        app_data.environment.data.insert(self.select_env_name.clone(), self.select_env_config.clone());
                                        self.select_env = None;
                                    }
                                }
                                if ui.button("Cancel").clicked() {
                                    self.select_env = None
                                }
                            }
                        });
                    });
            });
    }
}
