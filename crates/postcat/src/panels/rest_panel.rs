use egui::ahash::HashSet;
use egui::{Button, Label, RichText, Ui, Widget};
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::auth::{Auth, AuthType};
use crate::data::http::{BodyType, HttpRecord, LockWith, Method};
use crate::data::test::TestStatus;
use crate::data::workspace_data::WorkspaceData;
use crate::data::{http, test};
use crate::operation::operation::Operation;
use crate::panels::auth_panel::AuthPanel;
use crate::panels::request_body_panel::RequestBodyPanel;
use crate::panels::request_headers_panel::RequestHeadersPanel;
use crate::panels::request_params_panel::RequestParamsPanel;
use crate::panels::request_pre_script_panel::RequestPreScriptPanel;
use crate::panels::response_panel::ResponsePanel;
use crate::panels::test_script_panel::TestScriptPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::script::script::ScriptScope;
use crate::utils;
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;
use crate::windows::cookies_windows::CookiesWindows;
use crate::windows::save_crt_windows::SaveCRTWindows;

#[derive(Default)]
pub struct RestPanel {
    open_request_panel_enum: RequestPanelEnum,
    request_params_panel: RequestParamsPanel,
    auth_panel: AuthPanel,
    request_headers_panel: RequestHeadersPanel,
    request_body_panel: RequestBodyPanel,
    response_panel: ResponsePanel,
    request_pre_script_panel: RequestPreScriptPanel,
    test_script_panel: TestScriptPanel,
    send_promise:
        Option<Promise<Result<(http::Request, http::Response, test::TestResult), String>>>,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum RequestPanelEnum {
    Params,
    Authorization,
    Headers,
    Body,
    PreRequestScript,
    Tests,
}

impl Default for RequestPanelEnum {
    fn default() -> Self {
        RequestPanelEnum::Params
    }
}

impl RestPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(HORIZONTAL_GAP);
                self.render_name_label(workspace_data, crt_id.clone(), ui);
            });
            ui.separator();
            ui.horizontal(|ui| {
                self.render_editor_right_panel(operation, workspace_data, crt_id.clone(), ui);
                self.render_editor_left_panel(workspace_data, crt_id.clone(), ui);
            });
            ui.separator();
            self.render_middle_select(operation, workspace_data, crt_id.clone(), ui);
            ui.separator();
        });
        self.send_promise(ui, workspace_data, operation, crt_id.clone());
        self.render_request_open_panel(ui, operation, workspace_data, crt_id.clone());
        ui.separator();
        self.response_panel
            .set_and_render(ui, operation, workspace_data, crt_id.clone());
    }
    fn get_count(hr: &HttpRecord, panel_enum: RequestPanelEnum, parnet_auth: &Auth) -> usize {
        match panel_enum {
            RequestPanelEnum::Params => hr.request.params.iter().filter(|i| i.enable).count(),
            RequestPanelEnum::Authorization => {
                match hr.request.auth.get_final_type(parnet_auth.clone()) {
                    AuthType::InheritAuthFromParent => 0,
                    AuthType::NoAuth => 0,
                    AuthType::BearerToken => usize::MAX,
                    AuthType::BasicAuth => usize::MAX,
                }
            }
            RequestPanelEnum::Headers => hr.request.headers.iter().filter(|i| i.enable).count(),
            RequestPanelEnum::Body => match hr.request.body.body_type {
                BodyType::NONE => 0,
                BodyType::FROM_DATA => hr.request.body.body_form_data.len(),
                BodyType::X_WWW_FROM_URLENCODED => hr.request.body.body_xxx_form.len(),
                BodyType::RAW => {
                    if hr.request.body.body_str != "" {
                        usize::MAX
                    } else {
                        0
                    }
                }
                BodyType::BINARY => {
                    if hr.request.body.body_file != "" {
                        usize::MAX
                    } else {
                        0
                    }
                }
            },
            RequestPanelEnum::PreRequestScript => {
                if hr.pre_request_script != "" {
                    usize::MAX
                } else {
                    0
                }
            }
            RequestPanelEnum::Tests => {
                if hr.test_script != "" {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }

    fn render_name_label(
        &mut self,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
    ) {
        let envs = workspace_data.get_crt_envs(cursor.clone());
        let parent_auth = workspace_data.get_crt_parent_auth(cursor.clone());
        workspace_data.must_get_mut_crt(cursor.clone(), |crt| {
            crt.rest.sync_header(envs.clone(), parent_auth.clone());
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
        });
    }

    fn render_editor_right_panel(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
        ui: &mut Ui,
    ) {
        let mut send_rest = None;
        let client = workspace_data.build_http_client();
        let (mut pre_request_parent_script_scopes, mut test_parent_script_scopes) =
            workspace_data.get_crt_parent_scripts(crt_id.clone());
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let parent_auth = workspace_data.get_crt_parent_auth(crt_id.clone());
        let crt = workspace_data.must_get_crt(crt_id.clone());
        egui::SidePanel::right("editor_right_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP);
                    if self.send_promise.is_some() {
                        ui.add_enabled(false, Button::new("Send"));
                    } else {
                        if ui.button("Send").clicked() {
                            workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                crt.rest.prepare_send(envs.clone(), parent_auth.clone());
                            });
                            if crt.rest.pre_request_script.clone() != "" {
                                pre_request_parent_script_scopes.push(ScriptScope {
                                    scope: "request".to_string(),
                                    script: crt.rest.pre_request_script.clone(),
                                });
                            }

                            if crt.rest.test_script.clone() != "" {
                                test_parent_script_scopes.push(ScriptScope {
                                    scope: "request".to_string(),
                                    script: crt.rest.test_script.clone(),
                                });
                            }
                            let send_response = operation.send_with_script(
                                crt.rest.request.clone(),
                                envs.clone(),
                                pre_request_parent_script_scopes,
                                test_parent_script_scopes,
                                client,
                            );
                            self.send_promise = Some(send_response);
                            send_rest = Some(crt.rest.clone());
                        }
                    }
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
                                workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                    crt.set_baseline();
                                });
                            }
                        }
                    }
                });
            });

        send_rest.map(|r| {
            workspace_data.history_record(r);
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
                        egui::ComboBox::from_id_source("method")
                            .selected_text(crt.rest.request.method.clone().to_string())
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                for x in Method::iter() {
                                    ui.selectable_value(
                                        &mut crt.rest.request.method,
                                        x.clone(),
                                        x.to_string(),
                                    );
                                }
                            });
                        let mut filter: HashSet<String> = HashSet::default();
                        filter.insert(" ".to_string());
                        ui.centered_and_justified(|ui| {
                            let raw_url_text_edit = HighlightTemplateSinglelineBuilder::default()
                                .filter(filter)
                                .envs(envs.clone())
                                .all_space(false)
                                .build(cursor.clone() + "url", &mut crt.rest.request.raw_url)
                                .ui(ui);
                            if raw_url_text_edit.has_focus() {
                                crt.rest.sync_raw_url();
                            } else {
                                crt.rest.build_raw_url();
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
        let crt = workspace_data.must_get_crt(crt_id.clone());
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
                    workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                        self.auth_panel
                            .set_and_render(ui, &mut crt.rest.request.auth);
                    });
                }
            }
            RequestPanelEnum::Headers => {
                self.request_headers_panel
                    .set_and_render(ui, workspace_data, crt_id.clone())
            }
            RequestPanelEnum::Body => self.request_body_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                crt_id.clone(),
            ),

            RequestPanelEnum::PreRequestScript => {
                let script = self.request_pre_script_panel.set_and_render(
                    ui,
                    operation,
                    crt.rest.pre_request_script.clone(),
                    pre_request_parent_script_scopes,
                    crt.rest.request.clone(),
                    envs.clone(),
                    "rest".to_string(),
                );
                {
                    workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                        crt.rest.pre_request_script = script;
                    });
                }
            }
            RequestPanelEnum::Tests => {
                let script = self.test_script_panel.set_and_render(
                    ui,
                    crt.rest.test_script.clone(),
                    "rest".to_string(),
                );
                {
                    workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                        crt.rest.test_script = script;
                    });
                }
            }
        }
    }

    fn send_promise(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        crt_id: String,
    ) {
        if let Some(promise) = &self.send_promise {
            if let Some(result) = promise.ready() {
                workspace_data.save_cookies();
                workspace_data.must_get_mut_crt(crt_id.clone(), |crt| match result {
                    Ok((request, response, test_result)) => {
                        request
                            .headers
                            .iter()
                            .filter(|h| h.lock_with != LockWith::NoLock)
                            .for_each(|h| {
                                crt.rest.request.headers.push(h.clone());
                            });
                        crt.rest.response = response.clone();
                        crt.rest.ready();
                        operation.add_success_toast("Send request success");
                        crt.test_result = test_result.clone();
                        match test_result.status {
                            TestStatus::None => {}
                            TestStatus::PASS => {
                                operation.add_success_toast("Test success.");
                            }
                            TestStatus::FAIL => {
                                operation.add_error_toast("Test failed.");
                            }
                        }
                    }
                    Err(e) => {
                        crt.rest.error();
                        operation
                            .add_error_toast(format!("Send request failed: {}", e.to_string()));
                    }
                });
                self.send_promise = None;
            } else {
                ui.ctx().request_repaint();
                workspace_data.must_get_mut_crt(crt_id.clone(), |crt| crt.rest.pending());
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
                                RestPanel::get_count(&crt.rest, x, &parent_auth),
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
