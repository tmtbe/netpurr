use egui::Ui;
use poll_promise::Promise;
use strum::IntoEnumIterator;

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::panels::{HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;
use crate::windows::import_windows::ImportWindows;
use netpurr_core::data::workspace_data::{EditorModel, WorkspaceData};

#[derive(Default)]
pub struct TopPanel {
    current_workspace: String,
    sync_promise: Option<Promise<rustygit::types::Result<()>>>,
}

impl TopPanel {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
        config_data: &mut ConfigData,
    ) {
        ui.add_enabled_ui(!operation.get_ui_lock(), |ui| {
            // The top panel is often a good place for a menu bar:
            // egui::menu::bar(ui, |ui| {
            //     ui.menu_button("File", |ui| {
            //         if ui.button("New...").clicked() {}
            //         if ui.button("Import...").clicked() {
            //             operation.add_window(Box::new(ImportWindows::default()))
            //         }
            //         if ui.button("Exit").clicked() {
            //             ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            //         }
            //     });
            //     ui.menu_button("View", |ui| {
            //         if ui.button("Zoom In").clicked() {}
            //         if ui.button("Zoom Out").clicked() {}
            //     });
            //     egui::widgets::global_dark_light_mode_buttons(ui);
            // });
            egui::SidePanel::left("top_left_panel")
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(VERTICAL_GAP);
                        ui.horizontal(|ui| {
                            if ui.button("New").clicked() {}
                            ui.add_space(HORIZONTAL_GAP);
                            if ui.button("Import").clicked() {
                                operation.add_window(Box::new(ImportWindows::default()))
                            }
                        });
                        ui.add_space(VERTICAL_GAP);
                    });
                });
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    utils::add_left_space(ui, ui.available_width() / 2.0 - 80.0);
                    for editor_model in EditorModel::iter() {
                        ui.selectable_value(
                            &mut workspace_data.editor_model,
                            editor_model.clone(),
                            editor_model.to_string(),
                        );
                    }
                });
            });
        });
    }
}
