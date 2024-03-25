use eframe::emath::Align;
use egui::{Layout, Ui};

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::panels::HORIZONTAL_GAP;
use crate::utils;
use crate::widgets::matrix_label::{MatrixLabel, MatrixLabelType};
use crate::windows::workspace_windows::WorkspaceWindows;
use netpurr_core::data::workspace_data::WorkspaceData;

#[derive(Default)]
pub struct SelectedWorkspacePanel {
    current_workspace: Option<String>,
    selected_type: Option<String>,
}

impl SelectedWorkspacePanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
    ) {
        egui::SidePanel::left("selected_workspace_left_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                self.render_left(ui, workspace_data, config_data, operation);
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.strong("COLLECTIONS");
            if self.selected_type.is_some() && self.current_workspace.is_some() {
                ui.horizontal_wrapped(|ui| {
                    MatrixLabel::new(MatrixLabelType::Add).render(
                        ui,
                        workspace_data,
                        config_data,
                        operation,
                    );
                    ui.add_space(HORIZONTAL_GAP * 4.0);
                    let mut collection_names: Vec<String> =
                        workspace_data.get_collection_names().into_iter().collect();
                    collection_names.sort();
                    for name in collection_names {
                        MatrixLabel::new(MatrixLabelType::Collection(name.to_string())).render(
                            ui,
                            workspace_data,
                            config_data,
                            operation,
                        );
                        ui.add_space(HORIZONTAL_GAP * 4.0);
                    }
                });
            }
        });
    }

    fn render_left(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
        operation: &Operation,
    ) {
        egui::TopBottomPanel::top("workspace_top_panel")
            .exact_height(ui.available_height() / 2.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.strong(format!("WORKSPACES({})", config_data.workspaces().len()));
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .show(ui, |ui| {
                            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                if ui.button("⚙ Manage Workspace").clicked() {
                                    config_data.refresh_workspaces();
                                    operation.add_window(Box::new(
                                        WorkspaceWindows::default()
                                            .with_open(config_data.select_workspace()),
                                    ));
                                }
                            });
                            let workspaces = config_data.workspaces().clone();
                            for (name, _) in workspaces.iter() {
                                if self.current_workspace.is_none() {
                                    self.current_workspace = Some(name.clone());
                                    config_data.set_select_workspace(Some(name.clone()));
                                    workspace_data.load_all(name.clone());
                                }
                                if utils::select_value(
                                    ui,
                                    &mut self.current_workspace,
                                    Some(name.to_string()),
                                    name.to_string(),
                                )
                                .clicked()
                                {
                                    config_data.set_select_workspace(Some(name.clone()));
                                    workspace_data.load_all(name.clone());
                                }
                            }
                        });
                });
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.selected_type.is_none() {
                self.selected_type = Some("Collections".to_string())
            }
            utils::select_value(
                ui,
                &mut self.selected_type,
                Some("Collections".to_string()),
                format!(
                    "Collections({})",
                    workspace_data.get_collection_names().len()
                ),
            );
        });
    }
}
