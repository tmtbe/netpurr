use eframe::emath::Align;
use egui::{FontSelection, Label, RichText, Style, Ui, Widget};

use netpurr_core::data::record::Record;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::{EditorModel, WorkspaceData};
use crate::operation::operation::Operation;
use crate::panels::left_panel::MyLeftPanel;
use crate::panels::openapi_editor_panel::OpenApiEditorPanel;
use crate::panels::openapi_show_request_panel::OpenApiShowRequestPanel;
use crate::panels::response_panel::ResponsePanel;
use crate::panels::rest_panel::RestPanel;
use crate::panels::test_editor_panel::TestEditorPanel;
use crate::panels::websocket_panel::WebSocketPanel;
use crate::utils;
use crate::windows::request_close_windows::RequestCloseWindows;

#[derive(Default)]
pub struct SelectedCollectionPanel {
    rest_panel: RestPanel,
    left_panel: MyLeftPanel,
    web_socket_panel: WebSocketPanel,
    response_panel: ResponsePanel,
    select_crt_id: Option<String>,
    openapi_panel: OpenApiShowRequestPanel,
    openapi_editor_panel: OpenApiEditorPanel,
    test_editor_panel: TestEditorPanel,
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

impl SelectedCollectionPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
    ) {
        egui::SidePanel::left("selected_collection_left_panel")
            .default_width(220.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_enabled_ui(!operation.get_ui_lock(), |ui| {
                    self.left_panel
                        .set_and_render(ui, operation, workspace_data, config_data);
                });
            });
        egui::SidePanel::right("selected_collection_right_response")
            .min_width(ui.available_width() / 4.0)
            .max_width(ui.available_width() / 2.0)
            .show_inside(ui, |ui| {
                ui.add_enabled_ui(!operation.get_ui_lock(), |ui| {
                    match workspace_data.editor_model {
                        EditorModel::Request => {
                            self.render_request_response_panel(operation, workspace_data, ui);
                        }
                        EditorModel::Test => {
                            self.render_test_response_panel(operation, workspace_data, ui);
                        }
                        EditorModel::Design => {
                            workspace_data
                                .get_collection_by_name(
                                    config_data.select_collection().unwrap_or_default(),
                                )
                                .map(|c| {
                                    egui::scroll_area::ScrollArea::vertical()
                                        .max_height(ui.available_height() - 30.0)
                                        .show(ui, |ui| {
                                            self.openapi_panel.render(
                                                ui,
                                                workspace_data,
                                                operation,
                                                c,
                                            )
                                        });
                                });
                        }
                    }
                });
            });
        egui::CentralPanel::default().show_inside(ui, |ui| match workspace_data.editor_model {
            EditorModel::Request => {
                self.render_request_panel(operation, config_data, workspace_data, ui);
            }
            EditorModel::Test => {
                self.test_editor_panel.render(operation, workspace_data, ui);
            }
            EditorModel::Design => {
                self.openapi_editor_panel
                    .render(workspace_data, config_data, ui)
            }
        });
        workspace_data.get_env_select().map(|s| {
            if !workspace_data.get_env_configs().contains_key(s.as_str()) {
                workspace_data.set_env_select(None)
            }
        });
    }

    fn render_test_response_panel(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
    ) {
        match &workspace_data.selected_test_run_result.clone() {
            None => {}
            Some(test_run_result) => {
                self.response_panel.render_with_test(
                    ui,
                    operation,
                    workspace_data,
                    test_run_result,
                );
            }
        }
    }

    fn render_request_response_panel(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
    ) {
        match &workspace_data.get_crt_select_id() {
            None => {}
            Some(crt_id) => {
                self.response_panel
                    .render_with_crt(ui, operation, workspace_data, crt_id.clone());
            }
        }
    }

    fn render_request_panel(
        &mut self,
        operation: &Operation,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
    ) {
        ui.vertical(|ui| {
            self.central_request_table(workspace_data, operation, ui);
        });
        ui.separator();
        match &workspace_data.get_crt_select_id() {
            Some(crt_id) => {
                // ui.horizontal(|ui| {
                //     ui.add_space(HORIZONTAL_GAP);
                //     self.render_name_label(workspace_data, crt_id.clone(), ui);
                // });
                // ui.separator();
                match workspace_data.must_get_crt(crt_id.clone()).record {
                    Record::Rest(_) => {
                        self.rest_panel.set_and_render(
                            ui,
                            operation,
                            config_data,
                            workspace_data,
                            crt_id.clone(),
                        );
                    }
                    Record::WebSocket(_) => {
                        self.web_socket_panel.set_and_render(
                            ui,
                            operation,
                            config_data,
                            workspace_data,
                            crt_id.clone(),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn central_request_table(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        ui: &mut Ui,
    ) {
        ui.horizontal_wrapped(|ui| {
            let data_list = workspace_data.get_crt_id_list();
            for request_id in data_list.iter() {
                let request_data_option = workspace_data.get_crt_cloned(request_id.to_string());
                match request_data_option {
                    None => {
                        continue;
                    }
                    Some(request_data) => {
                        let mut lb =
                            utils::build_rest_ui_header(request_data.record.clone(), Some(15), ui);
                        if request_data.is_modify() {
                            let style = Style::default();
                            RichText::new(" ●")
                                .color(ui.visuals().warn_fg_color)
                                .append_to(&mut lb, &style, FontSelection::Default, Align::Center);
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
            ui.menu_button("+", |ui| {
                if ui.button("New Rest").clicked() {
                    workspace_data.add_new_rest_crt();
                    ui.close_menu();
                }
                if ui.button("New WebSocket").clicked() {
                    workspace_data.add_new_websocket_crt();
                    ui.close_menu();
                }
            });
            if ui.button("...").clicked() {}
        });
        let crt_list = workspace_data.get_crt_id_list();
        if crt_list.len() == 0 {
            workspace_data.add_new_rest_crt();
        }
        let crt_list = workspace_data.get_crt_id_list();
        if self.select_crt_id.is_none() {
            workspace_data.set_crt_select_id(crt_list.get(0).cloned());
        }
    }
    fn render_name_label(
        &mut self,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
        ui: &mut Ui,
    ) {
        let crt = workspace_data.must_get_crt(crt_id.clone());
        let tab_name = crt.get_tab_name();
        match &crt.collection_path {
            None => {
                ui.horizontal(|ui| {
                    ui.strong(tab_name);
                });
            }
            Some(collection_path) => {
                ui.horizontal(|ui| {
                    Label::new(
                        RichText::new(collection_path)
                            .strong()
                            .background_color(ui.visuals().extreme_bg_color),
                    )
                    .ui(ui);
                    ui.strong(tab_name);
                });
            }
        }
    }
}
