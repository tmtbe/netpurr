use eframe::epaint::ahash::HashSet;
use egui::{Ui, Widget};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use url::Url;

use netpurr_core::data::auth::{Auth, AuthType};
use netpurr_core::data::http::HttpRecord;
use netpurr_core::data::websocket::WebSocketStatus;

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::auth_panel::AuthPanel;
use crate::panels::request_headers_panel::RequestHeadersPanel;
use crate::panels::request_params_panel::RequestParamsPanel;
use crate::panels::request_pre_script_panel::RequestPreScriptPanel;
use crate::panels::websocket_content_panel::WebsocketContentPanel;
use crate::panels::websocket_event_panel::WebsocketEventPanel;
use crate::panels::HORIZONTAL_GAP;
use crate::utils;
use crate::utils::HighlightValue;
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;
use crate::windows::cookies_windows::CookiesWindows;
use crate::windows::save_crt_windows::SaveCRTWindows;

#[derive(Default)]
pub struct WebSocketPanel {
    websocket_content_panel: WebsocketContentPanel,
    open_request_panel_enum: RequestPanelEnum,
    request_params_panel: RequestParamsPanel,
    auth_panel: AuthPanel,
    request_headers_panel: RequestHeadersPanel,
    request_pre_script_panel: RequestPreScriptPanel,
    websocket_event_panel: WebsocketEventPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum RequestPanelEnum {
    Content,
    Params,
    Authorization,
    Headers,
    PreRequestScript,
}

impl Default for RequestPanelEnum {
    fn default() -> Self {
        RequestPanelEnum::Content
    }
}
impl WebSocketPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let parent_auth = workspace_data.get_crt_parent_auth(crt_id.clone());
        workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
            crt.record
                .must_get_mut_rest()
                .sync_everytime(envs.clone(), parent_auth.clone());
        });
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.render_editor_right_panel(operation, workspace_data, crt_id.clone(), ui);
                self.render_editor_left_panel(workspace_data, crt_id.clone(), ui);
            });
            ui.separator();
            self.render_middle_select(operation, workspace_data, crt_id.clone(), ui);
        });
        ui.separator();
        self.render_request_open_panel(ui, operation, workspace_data, crt_id.clone());
        self.websocket_event_panel
            .set_and_render(ui, operation, workspace_data, crt_id.clone());
        self.toast_event(operation, workspace_data, &crt_id);
    }
    fn get_count(
        hr: &HttpRecord,
        panel_enum: RequestPanelEnum,
        parent_auth: &Auth,
    ) -> HighlightValue {
        match panel_enum {
            RequestPanelEnum::Params => {
                HighlightValue::Usize(hr.request.params.iter().filter(|i| i.enable).count())
            }
            RequestPanelEnum::Authorization => {
                match hr.request.auth.get_final_type(parent_auth.clone()) {
                    AuthType::InheritAuthFromParent => HighlightValue::None,
                    AuthType::NoAuth => HighlightValue::None,
                    AuthType::BearerToken => HighlightValue::Has,
                    AuthType::BasicAuth => HighlightValue::Has,
                }
            }
            RequestPanelEnum::Headers => {
                HighlightValue::Usize(hr.request.headers.iter().filter(|i| i.enable).count())
            }
            RequestPanelEnum::Content => HighlightValue::None,
            RequestPanelEnum::PreRequestScript => {
                if hr.pre_request_script != "" {
                    HighlightValue::Has
                } else {
                    HighlightValue::None
                }
            }
        }
    }
    fn render_editor_right_panel(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
        ui: &mut Ui,
    ) {
        let (pre_request_parent_script_scopes, mut test_parent_script_scopes) =
            workspace_data.get_crt_parent_scripts(crt_id.clone());
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
        egui::SidePanel::right("editor_right_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP);
                    let mut connect = false;
                    let mut lock = false;
                    match &crt.record.must_get_websocket().session {
                        None => {
                            connect = false;
                        }
                        Some(session) => match session.get_status() {
                            WebSocketStatus::Connect => {
                                connect = true;
                            }
                            WebSocketStatus::Connecting => {
                                lock = true;
                            }
                            WebSocketStatus::Disconnect => {
                                connect = false;
                            }
                            WebSocketStatus::ConnectError(_) => {
                                connect = false;
                            }
                            WebSocketStatus::SendError(_) => {
                                connect = false;
                            }
                            WebSocketStatus::SendSuccess => {
                                connect = true;
                            }
                        },
                    }
                    ui.add_enabled_ui(!lock, |ui| {
                        if !connect {
                            if ui.button("Connect").clicked() {
                                println!("{}", crt.record.raw_url());
                                match Url::parse(crt.record.raw_url().as_str()) {
                                    Ok(url) => {
                                        crt = workspace_data.must_get_mut_crt(
                                            crt_id.clone(),
                                            |crt| {
                                                crt.record.must_get_mut_websocket().session =
                                                    Some(operation.connect_websocket_with_script(
                                                        crt.record.must_get_rest().request.clone(),
                                                        envs,
                                                        pre_request_parent_script_scopes,
                                                        test_parent_script_scopes,
                                                    ));
                                            },
                                        );
                                    }
                                    Err(e) => operation.add_error_toast(e.to_string()),
                                }
                                crt.record.must_get_websocket();
                            }
                        } else {
                            if ui.button("Disconnect").clicked() {
                                if let Some(session) = &crt.record.must_get_websocket().session {
                                    session.disconnect();
                                }
                            }
                        }
                    });
                    if ui.button("Save").clicked() {
                        match &crt.collection_path {
                            None => {
                                operation.add_window(Box::new(
                                    SaveCRTWindows::default().with(crt.id.clone()),
                                ));
                            }
                            Some(collection_path) => {
                                workspace_data.save_crt(
                                    crt.id.clone(),
                                    collection_path.clone(),
                                    |_| {},
                                );
                                operation.add_success_toast("Save success.");
                                crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                    crt.set_baseline();
                                });
                            }
                        }
                    }
                });
            });
    }

    fn toast_event(
        &self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: &String,
    ) {
        workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
            if let Some(session) = &crt.record.must_get_websocket().session {
                if let Some(event) = session.next_event() {
                    match event {
                        WebSocketStatus::Connect => {
                            crt.record.must_get_mut_rest().response = session.get_response();
                            operation.add_success_toast("Connected")
                        }
                        WebSocketStatus::Connecting => {}
                        WebSocketStatus::Disconnect => operation.add_success_toast("Disconnected"),
                        WebSocketStatus::ConnectError(e) => {
                            operation.add_error_toast(e.to_string())
                        }
                        WebSocketStatus::SendError(e) => operation.add_error_toast(e.to_string()),
                        WebSocketStatus::SendSuccess => {
                            operation.add_success_toast("Send message success.")
                        }
                    }
                }
            }
        });
    }
    fn render_editor_left_panel(
        &self,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
    ) {
        let envs = workspace_data.get_crt_envs(cursor.clone());
        workspace_data.must_get_mut_crt(cursor.clone(), |crt| {
            egui::SidePanel::left("editor_left_panel")
                .min_width(ui.available_width() - HORIZONTAL_GAP)
                .show_separator_line(false)
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut filter: HashSet<String> = HashSet::default();
                        filter.insert(" ".to_string());
                        ui.centered_and_justified(|ui| {
                            let raw_url_text_edit = HighlightTemplateSinglelineBuilder::default()
                                .filter(filter)
                                .envs(envs.clone())
                                .all_space(false)
                                .build(
                                    cursor.clone() + "url",
                                    &mut crt.record.must_get_mut_rest().request.raw_url,
                                )
                                .ui(ui);
                            if raw_url_text_edit.has_focus() {
                                crt.record.must_get_mut_rest().sync_raw_url();
                            } else {
                                crt.record.must_get_mut_rest().build_raw_url();
                            }
                        });
                    });
                });
        });
    }
    fn render_request_open_panel(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let (pre_request_parent_script_scopes, _) =
            workspace_data.get_crt_parent_scripts(crt_id.clone());
        match self.open_request_panel_enum {
            RequestPanelEnum::Params => {
                self.request_params_panel
                    .set_and_render(ui, workspace_data, crt_id.clone())
            }
            RequestPanelEnum::Authorization => {
                let mut parent_auth = None;
                match &crt.collection_path {
                    None => {}
                    Some(collection_path) => {
                        parent_auth =
                            Some(workspace_data.get_collection_auth(collection_path.clone()));
                    }
                }
                self.auth_panel.set_envs(envs.clone(), parent_auth);
                {
                    crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                        self.auth_panel
                            .set_and_render(ui, &mut crt.record.must_get_mut_rest().request.auth);
                    });
                }
            }
            RequestPanelEnum::Headers => {
                self.request_headers_panel
                    .set_and_render(ui, workspace_data, crt_id.clone())
            }
            RequestPanelEnum::Content => self.websocket_content_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                crt_id.clone(),
            ),
            RequestPanelEnum::PreRequestScript => {
                let script = self.request_pre_script_panel.set_and_render(
                    ui,
                    operation,
                    crt.get_tab_name(),
                    crt.record.pre_request_script(),
                    pre_request_parent_script_scopes,
                    crt.record.must_get_rest().request.clone(),
                    envs.clone(),
                    "rest".to_string(),
                );
                {
                    crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                        crt.record.set_pre_request_script(script);
                    });
                }
            }
        }
    }
    fn render_middle_select(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
        ui: &mut Ui,
    ) {
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
        let parent_auth = workspace_data.get_crt_parent_auth(crt_id.clone());
        utils::left_right_panel(
            ui,
            "rest_middle_select_label".to_string(),
            |ui| {
                ui.horizontal(|ui| {
                    for x in RequestPanelEnum::iter() {
                        ui.selectable_value(
                            &mut self.open_request_panel_enum,
                            x.clone(),
                            utils::build_with_count_ui_header(
                                x.to_string(),
                                Self::get_count(crt.record.must_get_rest(), x, &parent_auth),
                                ui,
                            ),
                        );
                    }
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    if ui.link("Cookies").clicked() {
                        operation.add_window(Box::new(CookiesWindows::default()));
                    };
                    ui.link("Code");
                });
            },
        );
    }
}
