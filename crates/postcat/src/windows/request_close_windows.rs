use egui::Ui;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::VERTICAL_GAP;
use crate::windows::save_windows::SaveWindows;

#[derive(Default)]
pub struct RequestCloseWindows {
    windows_open: bool,
    crt_id: String,
    tab_name: String,
}

impl Window for RequestCloseWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("DO YOU WANT TO SAVE?".to_string())
            .modal(true)
            .max_width(400.0)
            .min_height(400.0)
            .max_height(400.0)
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
        _: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        ui.label(format!("This tab `{}` has unsaved changes which will be lost if you choose to close it. Save these changes to avoid losing your work.", self.tab_name));
        ui.add_space(VERTICAL_GAP * 2.0);
        ui.horizontal(|ui| {
            if ui.button("Don't Save").clicked() {
                workspace_data.close_crt(self.crt_id.clone());
                self.set_open(false);
            }
            if ui.button("Cancel").clicked() {
                self.set_open(false);
            }
            ui.add_space(ui.available_width() - 100.0);
            if ui.button("Save change").clicked() {
                self.save_tab(workspace_data, &operation);
                self.set_open(false);
            }
        });
    }
}
impl RequestCloseWindows {
    pub fn with(mut self, crt_id: String, tab_name: String) -> Self {
        self.windows_open = true;
        self.crt_id = crt_id;
        self.tab_name = tab_name;
        self
    }
}

impl RequestCloseWindows {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        operation.lock_ui("request_close".to_string(), self.windows_open);
        let mut windows_open = self.windows_open;
        egui::Window::new("DO YOU WANT TO SAVE?")
            .default_open(true)
            .max_width(400.0)
            .min_height(400.0)
            .max_height(400.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut windows_open)
            .show(ui.ctx(), |ui| {
                ui.label(format!("This tab `{}` has unsaved changes which will be lost if you choose to close it. Save these changes to avoid losing your work.", self.tab_name));
                ui.add_space(VERTICAL_GAP * 2.0);
                ui.horizontal(|ui| {
                    if ui.button("Don't Save").clicked() {
                        workspace_data.close_crt(self.crt_id.clone());
                        self.windows_open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.windows_open = false;
                    }
                    ui.add_space(ui.available_width() - 100.0);
                    if ui.button("Save change").clicked() {
                        self.save_tab(workspace_data, operation);
                        self.windows_open = false;
                    }
                });
            });
        if self.windows_open {
            self.windows_open = windows_open;
        }
    }
    fn save_tab(&self, workspace_data: &mut WorkspaceData, operation: &Operation) {
        let crt_option = workspace_data.get_crt_cloned(self.crt_id.clone());
        crt_option.map(|crt| match &crt.collection_path {
            None => {
                operation.add_window(Box::new(SaveWindows::default().with(
                    crt.rest.clone(),
                    None,
                    false,
                )));
            }
            Some(collection_path) => {
                workspace_data.save_crt(self.crt_id.clone(), collection_path.clone(), |_| {});
                operation.add_success_toast("Save success.");
            }
        });
    }
}
