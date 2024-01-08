use eframe::emath::Align;
use egui::{FontSelection, Response, RichText, Style, Ui, WidgetText};

use crate::data::environment::ENVIRONMENT_GLOBALS;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::rest_panel::RestPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::utils;
use crate::windows::environment_windows::EnvironmentWindows;
use crate::windows::request_close_windows::RequestCloseWindows;

#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel: RestPanel,
    select_crt_id: Option<String>,
}

#[derive(PartialEq, Eq, Clone)]
enum PanelEnum {
    RequestId(Option<String>),
}

impl Default for PanelEnum {
    fn default() -> Self {
        PanelEnum::RequestId(None)
    }
}

impl DataView for MyCentralPanel {
    type CursorType = i32;
    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        ui.horizontal(|ui| {
            self.central_environment(workspace_data, operation, ui);
            self.central_request_table(workspace_data, operation, ui);
        });
        ui.separator();
        match &workspace_data.get_crt_select_id() {
            Some(request_id) => {
                self.editor_panel
                    .set_and_render(ui, operation, workspace_data, request_id.clone());
            }
            _ => {}
        }

        workspace_data.get_env_select().map(|s| {
            if !workspace_data.get_env_configs().contains_key(s.as_str()) {
                workspace_data.set_env_select(None)
            }
        });
    }
}

impl MyCentralPanel {
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

    fn central_environment(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        ui: &mut Ui,
    ) {
        egui::SidePanel::right("central_right_environment_panel")
            .resizable(true)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP * 2.0);
                    egui::ComboBox::from_id_source("env")
                        .selected_text(
                            workspace_data
                                .get_env_select()
                                .unwrap_or("No Environment".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            self.selectable_value(ui, workspace_data, None, "No Environment");
                            for (name, _) in &workspace_data.get_env_configs() {
                                if name == ENVIRONMENT_GLOBALS {
                                    continue;
                                }
                                self.selectable_value(
                                    ui,
                                    workspace_data,
                                    Some(name.clone()),
                                    name.clone(),
                                );
                            }
                        });
                    if ui.button("⚙").clicked() {
                        operation.add_window(Box::new(EnvironmentWindows::default()));
                    }
                });
            });
    }

    fn central_request_table(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        ui: &mut Ui,
    ) {
        egui::SidePanel::left("central_request_table_panel")
            .resizable(true)
            .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
            .show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let data_list = workspace_data.get_crt_id_list();
                    for request_id in data_list.iter() {
                        let request_data_option =
                            workspace_data.get_crt_cloned(request_id.to_string());
                        match request_data_option {
                            None => {
                                continue;
                            }
                            Some(request_data) => {
                                let mut lb = utils::build_rest_ui_header(
                                    request_data.rest.clone(),
                                    Some(15),
                                    ui,
                                );
                                if request_data.is_modify() {
                                    let style = Style::default();
                                    RichText::new(" ●")
                                        .color(ui.visuals().warn_fg_color)
                                        .append_to(
                                            &mut lb,
                                            &style,
                                            FontSelection::Default,
                                            Align::Center,
                                        );
                                }
                                self.select_crt_id = workspace_data.get_crt_select_id();
                                let response = ui.selectable_value(
                                    &mut self.select_crt_id,
                                    Some(request_data.id.clone()),
                                    lb,
                                );
                                if response.clicked() {
                                    workspace_data.set_crt_select_id(self.select_crt_id.clone());
                                }
                                response.context_menu(|ui| {
                                    if ui.button("Duplicate Tab").clicked() {
                                        workspace_data.duplicate_crt(request_data.id.clone());
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button("Close").clicked() {
                                        if !request_data.is_modify() {
                                            workspace_data.close_crt(request_data.id.clone());
                                        } else {
                                            operation.add_window(Box::new(
                                                RequestCloseWindows::default().with(
                                                    request_data.id.clone(),
                                                    request_data.get_tab_name(),
                                                ),
                                            ))
                                        }
                                        ui.close_menu();
                                    }
                                    if ui.button("Force Close").clicked() {
                                        workspace_data.close_crt(request_data.id.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("Force Close Other Tabs").clicked() {
                                        workspace_data.close_other_crt(request_data.id.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("Force Close All Tabs").clicked() {
                                        workspace_data.close_all_crt();
                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    }
                    if ui.button("+").clicked() {
                        workspace_data.add_new_crt();
                    }
                    if ui.button("...").clicked() {}
                });
            });
        let crt_list = workspace_data.get_crt_id_list();
        if crt_list.len() == 0 {
            workspace_data.add_new_crt();
        }
        let crt_list = workspace_data.get_crt_id_list();
        if self.select_crt_id.is_none() {
            workspace_data.set_crt_select_id(crt_list.get(0).cloned());
        }
    }
}
