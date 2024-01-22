use egui::Ui;
use poll_promise::Promise;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::HORIZONTAL_GAP;
use crate::windows::import_windows::ImportWindows;

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
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New...").clicked() {}
                    if ui.button("Import...").clicked() {
                        operation.add_window(Box::new(ImportWindows::default()))
                    }
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In").clicked() {}
                    if ui.button("Zoom Out").clicked() {}
                });
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
            ui.add_space(HORIZONTAL_GAP);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New").clicked() {}
                    ui.add_space(HORIZONTAL_GAP);
                    if ui.button("Import").clicked() {
                        operation.add_window(Box::new(ImportWindows::default()))
                    }
                });
            });
            ui.add_space(HORIZONTAL_GAP);
        });
    }
}
