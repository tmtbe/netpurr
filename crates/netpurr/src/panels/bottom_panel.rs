use egui::{Response, Ui, WidgetText};
use poll_promise::Promise;

use netpurr_core::data::environment::ENVIRONMENT_GLOBALS;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::VERTICAL_GAP;
use crate::utils;
use crate::windows::environment_windows::EnvironmentWindows;
use crate::windows::workspace_windows::WorkspaceWindows;

#[derive(Default)]
pub struct BottomPanel {
    current_workspace: String,
    environment: String,
    sync_promise: Option<Promise<rustygit::types::Result<()>>>,
}

const NO_ENVIRONMENT: &str = "No Environment";

impl BottomPanel {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
        config_data: &mut ConfigData,
    ) {
        ui.add_enabled_ui(!operation.get_ui_lock(), |ui| {
            ui.add_space(VERTICAL_GAP * 2.0);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("environment")
                    .selected_text(
                        workspace_data
                            .get_env_select()
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
                if self.environment == NO_ENVIRONMENT {
                    workspace_data.set_env_select(None);
                } else {
                    workspace_data.set_env_select(Some(self.environment.to_string()));
                }
                self.current_workspace = config_data.select_workspace().to_string();
                egui::ComboBox::from_id_source("workspace")
                    .selected_text("Workspace: ".to_string() + self.current_workspace.as_str())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        if ui.button("⚙ Manage Workspace").clicked() {
                            config_data.refresh_workspaces();
                            let current_workspace = config_data.select_workspace().to_string();
                            operation.add_window(Box::new(
                                WorkspaceWindows::default().with_open(current_workspace),
                            ));
                        }
                        for (name, _) in config_data.workspaces().iter() {
                            ui.selectable_value(
                                &mut self.current_workspace,
                                name.to_string(),
                                name.to_string(),
                            );
                        }
                    });
                let select_workspace = config_data.select_workspace().to_string();
                if let Some(workspace) = config_data
                    .mut_workspaces()
                    .get_mut(select_workspace.as_str())
                {
                    if workspace.if_enable_git() && workspace.if_enable_git() {
                        if self.sync_promise.is_some() {
                            ui.add_enabled_ui(false, |ui| ui.button("🔄"));
                        } else {
                            if ui.button("🔄").clicked() {
                                self.sync_promise =
                                    Some(operation.git().git_sync_promise(workspace.path.clone()));
                            }
                        }
                    }
                }
                match &self.sync_promise {
                    None => {}
                    Some(result) => match result.ready() {
                        None => {
                            ui.ctx().request_repaint();
                        }
                        Some(result) => {
                            if result.is_ok() {
                                operation.add_success_toast("Sync Success!")
                            } else {
                                operation.add_error_toast("Sync Failed!")
                            }
                            self.sync_promise = None;
                            workspace_data.reload_data(self.current_workspace.clone());
                        }
                    },
                }
                if self.current_workspace != config_data.select_workspace() {
                    config_data.set_select_workspace(self.current_workspace.clone());
                    workspace_data.load_all(self.current_workspace.clone());
                }
                utils::add_right_space(ui, 280.0);
                ui.label("Made with jincheng.zhang@thoughtworks.com");
            });
        });
    }
    pub fn selectable_value(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        selected_value: Option<String>,
        text: impl Into<WidgetText>,
    ) -> Response {
        let mut response =
            ui.selectable_label(workspace_data.get_env_select() == selected_value, text);
        if response.clicked() && workspace_data.get_env_select() != selected_value {
            workspace_data.set_env_select(selected_value);
            response.mark_changed();
        }
        response
    }
}
