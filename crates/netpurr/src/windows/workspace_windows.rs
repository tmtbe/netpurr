use std::path::PathBuf;

use egui::{Spinner, Ui, Widget};
use poll_promise::Promise;
use rustygit::Repository;

use netpurr_core::data::workspace_data::WorkspaceData;

use crate::data::config_data::ConfigData;
use crate::data::workspace::Workspace;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::HORIZONTAL_GAP;
use crate::utils;

#[derive(Default)]
pub struct WorkspaceWindows {
    windows_open: bool,
    current_workspace: Option<String>,
    current_workspace_git_repo_name: String,
    current_workspace_git_repo: Option<PathBuf>,
    current_branch_list: Vec<String>,
    user_git_branch: String,
    user_git_remote_url: String,
    git_branch: Option<String>,
    git_remote_url: Option<String>,
    git_remote_edit: bool,
    new_workspace_name: String,
    sync_promise: Option<Promise<rustygit::types::Result<()>>>,
    force_pull_promise: Option<Promise<rustygit::types::Result<()>>>,
    force_push_promise: Option<Promise<rustygit::types::Result<()>>>,
    status: String,
}

impl Window for WorkspaceWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("MANAGE WORKSPACE")
            .modal(true)
            .default_width(500.0)
            .default_height(300.0)
            .collapsible(false)
            .resizable(true)
    }

    fn set_open(&mut self, open: bool) {
        self.windows_open = open;
    }

    fn get_open(&self) -> bool {
        self.windows_open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        config_data: &mut ConfigData,
        _: &mut WorkspaceData,
        operation: Operation,
    ) {
        self.render_left_panel(ui, config_data);
        let option_workspace = config_data
            .workspaces()
            .get(self.current_workspace.clone().unwrap_or_default().as_str());
        option_workspace.map(|workspace| {
            self.update(workspace, &operation);
            ui.horizontal(|ui| {
                ui.add_space(HORIZONTAL_GAP);
                egui::ScrollArea::vertical()
                    .max_width(500.0)
                    .min_scrolled_height(500.0)
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
                            ui.separator();
                            match &self.current_workspace_git_repo {
                                None => {
                                    if ui.button("Enable Git").clicked() {
                                        operation.git().enable_git(&workspace.path);
                                    }
                                }
                                Some(_) => {
                                    self.render_branch(ui, &operation, &workspace.path);
                                    self.render_remote(ui, &operation, &workspace.path);
                                    if self.git_branch.is_some() && self.git_remote_url.is_some() {
                                        ui.horizontal(|ui| {
                                            let lock = self.sync_promise.is_some()
                                                || self.force_pull_promise.is_some()
                                                || self.force_push_promise.is_some();
                                            ui.add_enabled_ui(!lock, |ui| {
                                                self.sync_button(ui, workspace, &operation);
                                                self.force_pull_button(ui, workspace, &operation);
                                                self.force_push(ui, workspace, &operation);
                                            });
                                        });
                                    }
                                }
                            }
                            ui.separator();
                            ui.label(self.status.clone())
                        });
                    })
            });
        });
    }
}

impl WorkspaceWindows {
    fn force_push(&mut self, ui: &mut Ui, workspace: &Workspace, operation: &Operation) {
        let button = ui.button("Force Push");
        button.clone().on_hover_text(
            "Force local push to remote, used when synchronization conflicts occur.",
        );
        if button.clicked() {
            self.status = "Waiting ...".to_string();
            self.force_push_promise = Some(
                operation
                    .git()
                    .git_force_push_promise(workspace.path.clone()),
            );
        }
        if let Some(promise) = &self.force_push_promise {
            Spinner::new().ui(ui);
            if let Some(result) = promise.ready() {
                match result {
                    Ok(_) => self.status = "Force Push Success.".to_string(),
                    Err(e) => {
                        self.status = format!("Force Push Failed: {}", e.to_string());
                    }
                }
                self.force_push_promise = None;
            }
        }
    }

    fn force_pull_button(&mut self, ui: &mut Ui, workspace: &Workspace, operation: &Operation) {
        let button = ui.button("Force Pull");
        button.clone().on_hover_text(
            "Force the remote data to be pulled down, ignore local submission, and be used when synchronization conflicts occur.",
        );
        if button.clicked() {
            self.status = "Waiting ...".to_string();
            self.force_pull_promise = Some(
                operation
                    .git()
                    .git_force_pull_promise(workspace.path.clone()),
            );
        }
        if let Some(promise) = &self.force_pull_promise {
            Spinner::new().ui(ui);
            if let Some(result) = promise.ready() {
                match result {
                    Ok(_) => self.status = "Force Pull Success.".to_string(),
                    Err(e) => {
                        self.status = format!("Force Pull Failed: {}", e.to_string());
                    }
                }
                self.force_pull_promise = None;
            }
        }
    }

    fn sync_button(&mut self, ui: &mut Ui, workspace: &Workspace, operation: &Operation) {
        let button = ui.button("Sync");
        button.clone().on_hover_text(
            "Synchronize data to remote git, it will automatically `commit`, `rebase` and `push`",
        );
        if button.clicked() {
            self.status = "Waiting ...".to_string();
            self.sync_promise = Some(operation.git().git_sync_promise(workspace.path.clone()));
        }
        if let Some(promise) = &self.sync_promise {
            Spinner::new().ui(ui);
            if let Some(result) = promise.ready() {
                match result {
                    Ok(_) => self.status = "Sync Success.".to_string(),
                    Err(e) => {
                        self.status = format!("Sync Failed: {}", e.to_string());
                    }
                }
                self.sync_promise = None;
            }
        }
    }

    fn render_left_panel(&mut self, ui: &mut Ui, config_data: &mut ConfigData) {
        egui::SidePanel::left("workspace_left_panel")
            .default_width(150.0)
            .width_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if config_data
                            .workspaces()
                            .contains_key(self.new_workspace_name.as_str())
                        {
                            ui.add_enabled_ui(false, |ui| {
                                ui.button("+");
                            });
                        } else {
                            if ui.button("+").clicked() {
                                config_data.new_workspace(self.new_workspace_name.clone());
                            }
                        }
                        utils::text_edit_singleline_filter_justify(
                            ui,
                            &mut self.new_workspace_name,
                        );
                    });

                    for (name, _) in config_data.workspaces().iter() {
                        if utils::select_value(
                            ui,
                            &mut self.current_workspace,
                            Some(name.to_string()),
                            name.to_string(),
                        )
                        .clicked()
                        {
                            self.user_git_branch = "main".to_string();
                            self.user_git_remote_url = "".to_string();
                            self.git_branch = None;
                            self.git_remote_url = None;
                            self.current_workspace_git_repo = None;
                            self.current_workspace_git_repo_name = "".to_string();
                            self.status = "".to_string();
                        }
                    }
                });
            });
    }
    fn render_remote(&mut self, ui: &mut Ui, operation: &Operation, path: &PathBuf) {
        ui.horizontal(|ui| {
            ui.strong("Git Origin Url:");
            if !self.git_remote_edit {
                if ui.button("⚙").clicked() {
                    self.git_remote_edit = true;
                    self.git_remote_url.clone().map(|r| {
                        self.user_git_remote_url = r.clone();
                    });
                }
                ui.label(&self.user_git_remote_url);
            } else {
                if ui.button("✔").clicked() {
                    self.git_remote_edit = false;
                    if self.user_git_remote_url != "" {
                        operation.git().update_remote(path, self.user_git_remote_url.clone());
                    }
                }
                utils::text_edit_singleline_justify(ui, &mut self.user_git_remote_url)
                    .on_hover_text("Since Netpurr uses local git tools, it is recommended to use `ssh` to set the git address to prevent errors.");
            }
        });
    }
    fn render_branch(&mut self, ui: &mut Ui, operation: &Operation, path: &PathBuf) {
        match &self.git_branch {
            None => {
                ui.horizontal(|ui| {
                    ui.strong("Git Branch");
                    utils::text_edit_singleline_filter_justify(ui, &mut self.user_git_branch);
                });
                if ui.button("Create Branch").clicked() {
                    operation
                        .git()
                        .create_branch(path, self.user_git_branch.clone());
                };
            }
            Some(branch_name) => {
                ui.horizontal(|ui| {
                    ui.strong("Switch Git Branch:");
                    egui::ComboBox::from_id_source("branch")
                        .selected_text(branch_name)
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            for select_branch in &self.current_branch_list {
                                if ui
                                    .selectable_value(
                                        &mut self.user_git_branch,
                                        select_branch.clone(),
                                        select_branch.to_string(),
                                    )
                                    .clicked()
                                {
                                    operation
                                        .git()
                                        .switch_branch(path, self.user_git_branch.clone());
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    utils::text_edit_singleline_filter(ui, &mut self.user_git_branch);
                    let button = ui.button("Create Local Branch");
                    button.clone().on_hover_text("Create a local branch. The local branch and the remote branch have a one-to-one correspondence.");
                    if button.clicked() {
                        operation.git().create_branch(path, self.user_git_branch.clone());
                    };
                });
            }
        }
    }

    pub fn with_open(mut self, current_workspace: Option<String>) -> Self {
        self.windows_open = true;
        self.user_git_branch = "main".to_string();
        self.current_workspace = current_workspace;
        self
    }

    fn update(&mut self, workspace: &Workspace, operation: &Operation) {
        if self.current_workspace_git_repo_name
            != self.current_workspace.clone().unwrap_or_default()
        {
            if operation.git().if_enable_git(&workspace.path) {
                self.current_workspace_git_repo = Some(workspace.path.clone());
            } else {
                self.current_workspace_git_repo = None;
            }
        }
        match &self.current_workspace_git_repo {
            None => {}
            Some(repo_path) => {
                let repo = Repository::new(repo_path);
                if let Ok(branches) = repo.list_branches() {
                    self.current_branch_list = branches.clone();
                    if let Ok(head) = repo.cmd_out(["branch", "--show-current"]) {
                        if head.len() > 0 {
                            let branch = head[0].to_string();
                            if branches.contains(&branch) {
                                self.git_branch = Some(branch);
                            }
                        }
                    }
                }

                if let Ok(remote) = repo.cmd_out(["remote", "get-url", "origin"]) {
                    if remote.len() > 0 {
                        self.git_remote_url = Some(remote[0].clone());
                        if self.user_git_remote_url == "" {
                            self.user_git_remote_url = remote[0].clone();
                        }
                    }
                }
            }
        }
    }
}
