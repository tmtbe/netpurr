use eframe::emath::Align;
use egui::{FontSelection, Label, RichText, Style, Ui, Widget};

use netpurr_core::data::record::Record;

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::rest_panel::RestPanel;
use crate::panels::websocket_panel::WebSocketPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::utils;
use crate::windows::request_close_windows::RequestCloseWindows;

#[derive(Default)]
pub struct MyCentralPanel {
    rest_panel: RestPanel,
    web_socket_panel: WebSocketPanel,
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

impl MyCentralPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        self.central_request_table(workspace_data, operation, ui);
        ui.separator();
        match &workspace_data.get_crt_select_id() {
            Some(crt_id) => {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP);
                    self.render_name_label(workspace_data, crt_id.clone(), ui);
                });
                ui.separator();
                match workspace_data.must_get_crt(crt_id.clone()).record {
                    Record::Rest(_) => {
                        self.rest_panel.set_and_render(
                            ui,
                            operation,
                            workspace_data,
                            crt_id.clone(),
                        );
                    }
                    Record::WebSocket(_) => {
                        self.web_socket_panel.set_and_render(
                            ui,
                            operation,
                            workspace_data,
                            crt_id.clone(),
                        );
                    }
                }
            }
            _ => {}
        }

        workspace_data.get_env_select().map(|s| {
            if !workspace_data.get_env_configs().contains_key(s.as_str()) {
                workspace_data.set_env_select(None)
            }
        });
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
