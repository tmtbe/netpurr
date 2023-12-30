use egui::Ui;

use crate::data::ConfigData;
use crate::operation::Operation;
use crate::panels::HORIZONTAL_GAP;
use crate::utils;

#[derive(Default)]
pub struct WorkspaceWindows {
    environment_windows_open: bool,
    current_workspace: String,
}

impl WorkspaceWindows {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        config_data: &mut ConfigData,
    ) {
        operation.lock_ui("workspace".to_string(), self.environment_windows_open);
        egui::Window::new("MANAGE WORKSPACE")
            .default_open(true)
            .default_width(500.0)
            .default_height(300.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut self.environment_windows_open)
            .show(ui.ctx(), |ui| {
                egui::SidePanel::left("workspace_left_panel")
                    .default_width(150.0)
                    .width_range(80.0..=200.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (name, _) in config_data.workspaces().iter() {
                                utils::select_value(
                                    ui,
                                    &mut self.current_workspace,
                                    name.to_string(),
                                    name.to_string(),
                                );
                            }
                        });
                    });
                let option_workspace = config_data
                    .workspaces()
                    .get(self.current_workspace.as_str());
                option_workspace.map(|workspace| {
                    ui.horizontal(|ui| {
                        ui.add_space(HORIZONTAL_GAP);
                        egui::ScrollArea::vertical()
                            .max_width(300.0)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong("Name: ");
                                        let mut name = workspace.name.as_str();
                                        utils::text_edit_singleline_justify(ui, &mut name);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.strong("Path: ");
                                        let mut path = workspace.path.to_str().unwrap_or("");
                                        utils::text_edit_singleline_justify(ui, &mut path);
                                    });
                                });
                            })
                    });
                })
            });
    }
    pub fn open(&mut self, config_data: &ConfigData) {
        self.environment_windows_open = true;
        self.current_workspace = config_data.select_workspace().to_string();
    }
}
