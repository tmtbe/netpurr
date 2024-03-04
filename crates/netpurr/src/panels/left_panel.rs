use eframe::emath::Align;
use egui::{Label, Layout, ScrollArea, Sense, Widget};

use netpurr_core::data::environment::ENVIRONMENT_GLOBALS;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::collection_panel::CollectionPanel;
use crate::panels::history_panel::HistoryPanel;
use crate::panels::VERTICAL_GAP;
use crate::windows::cookies_windows::CookiesWindows;
use crate::windows::environment_windows::EnvironmentWindows;
use crate::windows::new_collection_windows::NewCollectionWindows;

#[derive(PartialEq, Eq)]
enum Panel {
    History,
    Collection,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Collection
    }
}

#[derive(Default)]
pub struct MyLeftPanel {
    history_panel: HistoryPanel,
    collection_panel: CollectionPanel,
    open_panel: Panel,
    filter: String,
    environment: String,
}

const NO_ENVIRONMENT: &str = "No Environment";
impl MyLeftPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
    ) {
        let collection = workspace_data
            .get_collection_by_name(config_data.select_collection().unwrap_or_default());
        egui::TopBottomPanel::top("left_top_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if Label::new("<").sense(Sense::click()).ui(ui).clicked() {
                        config_data.set_select_collection(None);
                    }
                    ui.separator();
                    if ui
                        .link(config_data.select_collection().unwrap_or_default())
                        .clicked()
                    {
                        operation.add_window(Box::new(
                            NewCollectionWindows::default().with_open_collection(collection),
                        ));
                    }
                });
                ui.separator();
                egui::ComboBox::from_id_source("environment")
                    .width(ui.available_width())
                    .selected_text(
                        workspace_data
                            .get_env_select()
                            .map(|name| "Environment: ".to_string() + name.as_str())
                            .unwrap_or(NO_ENVIRONMENT.to_string()),
                    )
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        if ui.button("⚙ Manage Environment").clicked() {
                            operation.add_window(Box::new(EnvironmentWindows::default()));
                        }
                        match workspace_data.get_env_select() {
                            None => {
                                self.environment = NO_ENVIRONMENT.to_string();
                            }
                            Some(env) => {
                                self.environment = env;
                            }
                        }
                        ui.selectable_value(
                            &mut self.environment,
                            NO_ENVIRONMENT.to_string(),
                            NO_ENVIRONMENT.to_string(),
                        );
                        for (name, _) in &workspace_data.get_env_configs() {
                            if name == ENVIRONMENT_GLOBALS {
                                continue;
                            }
                            ui.selectable_value(&mut self.environment, name.clone(), name.clone());
                        }
                    });
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    if ui.button("Manager Cookies").clicked() {
                        operation.add_window(Box::new(CookiesWindows::default()));
                    }
                });

                ui.add_space(VERTICAL_GAP / 8.0);
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.open_panel, Panel::Collection, "Collection");
                ui.selectable_value(&mut self.open_panel, Panel::History, "History");
            });
            ScrollArea::vertical().show(ui, |ui| match self.open_panel {
                Panel::History => {
                    self.history_panel.set_and_render(ui, workspace_data);
                }
                Panel::Collection => {
                    self.collection_panel.set_and_render(
                        ui,
                        operation,
                        workspace_data,
                        config_data.select_collection().unwrap_or_default(),
                    );
                }
            });
        });

        if self.environment == NO_ENVIRONMENT {
            workspace_data.set_env_select(None);
        } else {
            workspace_data.set_env_select(Some(self.environment.to_string()));
        }
    }
}
