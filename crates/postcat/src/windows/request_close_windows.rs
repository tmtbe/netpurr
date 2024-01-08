use egui::Ui;
use egui_toast::{Toast, ToastKind};

use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::VERTICAL_GAP;

#[derive(Default)]
pub struct RequestCloseWindows {
    windows_open: bool,
    crt_id: String,
    tab_name: String,
}

impl RequestCloseWindows {
    pub fn open(&mut self, crt_id: String, tab_name: String) {
        self.windows_open = true;
        self.crt_id = crt_id;
        self.tab_name = tab_name;
    }
}

impl RequestCloseWindows {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
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
    fn save_tab(&self, workspace_data: &mut WorkspaceData, operation: &mut Operation) {
        let crt_option = workspace_data.get_crt_cloned(self.crt_id.clone());
        crt_option.map(|crt| match &crt.collection_path {
            None => {
                operation.open_windows().open_save(crt.rest.clone(), None);
            }
            Some(collection_path) => {
                workspace_data.save_crt(self.crt_id.clone(), collection_path.clone(), |_| {});
                operation.toasts().add(Toast {
                    kind: ToastKind::Success,
                    text: "Save success.".into(),
                    options: Default::default(),
                });
            }
        });
    }
}
