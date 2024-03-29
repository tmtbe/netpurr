use egui::{Button, Ui, Widget};
use egui::ahash::HashSet;
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use netpurr_core::data::auth::{Auth, AuthType};
use netpurr_core::data::http::{BodyType, HttpRecord, LockWith, Method};
use netpurr_core::data::test::TestStatus;
use netpurr_core::data::workspace_data::WorkspaceData;
use netpurr_core::runner::{RunRequestInfo, TestRunError, TestRunResult};

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::panels::auth_panel::AuthPanel;
use crate::panels::HORIZONTAL_GAP;
use crate::panels::request_body_panel::RequestBodyPanel;
use crate::panels::request_headers_panel::RequestHeadersPanel;
use crate::panels::request_params_panel::RequestParamsPanel;
use crate::panels::request_pre_script_panel::RequestPreScriptPanel;
use crate::panels::test_script_panel::TestScriptPanel;
use crate::utils;
use crate::utils::HighlightValue;
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;
use crate::windows::save_crt_windows::SaveCRTWindows;

#[derive(Default)]
pub struct RestPanel {
    open_request_panel_enum: RequestPanelEnum,
    request_params_panel: RequestParamsPanel,
    auth_panel: AuthPanel,
    request_headers_panel: RequestHeadersPanel,
    request_body_panel: RequestBodyPanel,
    request_pre_script_panel: RequestPreScriptPanel,
    test_script_panel: TestScriptPanel,
    send_promise: Option<Promise<Result<TestRunResult, TestRunError>>>,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum RequestPanelEnum {
    Params,
    Authorization,
    Headers,
    Body,
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
        config_data: &mut ConfigData,
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
                self.render_editor_right_panel(
                    operation,
                    config_data,
                    workspace_data,
                    crt_id.clone(),
                    ui,
                );
                self.render_editor_left_panel(workspace_data, crt_id.clone(), ui);
            });
            ui.separator();
            self.render_middle_select(operation, workspace_data, crt_id.clone(), ui);
            ui.separator();
        });
        self.send_promise(ui, workspace_data, operation, crt_id.clone());
        egui::ScrollArea::horizontal().show(ui, |ui| {
            self.render_request_open_panel(ui, operation, workspace_data, crt_id.clone());
        });
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
            RequestPanelEnum::Body => match hr.request.body.body_type {
                BodyType::NONE => HighlightValue::None,
                BodyType::FROM_DATA => HighlightValue::Usize(hr.request.body.body_form_data.len()),
                BodyType::X_WWW_FROM_URLENCODED => {
                    HighlightValue::Usize(hr.request.body.body_xxx_form.len())
                }
                BodyType::RAW => {
                    if hr.request.body.body_str != "" {
                        HighlightValue::Has
                    } else {
                        HighlightValue::None
                    }
                }
                BodyType::BINARY => {
                    if hr.request.body.body_file != "" {
                        HighlightValue::Has
                    } else {
                        HighlightValue::None
                    }
                }
            },
        }
    }

    fn render_editor_right_panel(
        &mut self,
        operation: &Operation,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
        ui: &mut Ui,
    ) {
        let mut send_rest = None;
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let parent_auth = workspace_data.get_crt_parent_auth(crt_id.clone());
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
        egui::SidePanel::right("editor_right_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP);
                    if self.send_promise.is_some() {
                        ui.add_enabled(false, Button::new("Send"));
                    } else {
                        if ui.button("Send").clicked() {
                            crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                crt.record
                                    .must_get_mut_rest()
                                    .prepare_send(envs.clone(), parent_auth.clone());
                            });
                            let send_response =
                                operation.send_rest_with_script_promise(RunRequestInfo {
                                    shared_map: Default::default(),
                                    collection_path: crt.collection_path.clone(),
                                    request_name: crt.get_tab_name(),
                                    request: crt.record.must_get_rest().request.clone(),
                                    envs: envs.clone(),
                                    pre_request_scripts: vec![],
                                    test_scripts: vec![],
                                    testcase: Default::default(),
                                });
                            self.send_promise = Some(send_response);
                            send_rest = Some(crt.record.clone());
                        }
                    }
                    if ui.button("Save").clicked() {
                        match &crt.collection_path {
                            None => {
                                operation.add_window(Box::new(SaveCRTWindows::default().with(
                                    crt.id.clone(),
                                    config_data.select_collection().clone(),
                                )));
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
            let mut min_width = ui.available_width() - HORIZONTAL_GAP;
            if min_width < HORIZONTAL_GAP {
                min_width = HORIZONTAL_GAP
            }
            egui::SidePanel::left("editor_left_panel")
                .min_width(min_width)
                .show_separator_line(false)
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_source("method")
                            .selected_text(crt.record.method())
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                for x in Method::iter() {
                                    ui.selectable_value(
                                        &mut crt.record.must_get_mut_rest().request.method,
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
                    .set_and_render(ui, workspace_data, crt_id.clone());
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
                    .set_and_render(ui, workspace_data, crt_id.clone());
            }
            RequestPanelEnum::Body => self.request_body_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                crt_id.clone(),
            ),
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
                    Ok(test_run_result) => {
                        test_run_result
                            .request
                            .headers
                            .iter()
                            .filter(|h| h.lock_with != LockWith::NoLock)
                            .for_each(|h| {
                                crt.record
                                    .must_get_mut_rest()
                                    .request
                                    .headers
                                    .push(h.clone());
                            });
                        match &test_run_result.response {
                            None => {
                                crt.record.must_get_mut_rest().error();
                                operation.add_error_toast("Send request failed: Response is none".to_string());
                            }
                            Some(response) => {
                                crt.record.must_get_mut_rest().response = response.clone();
                                crt.record.must_get_mut_rest().ready();
                                operation.add_success_toast("Send request success");
                                crt.test_result = test_run_result.test_result.clone();
                                match test_run_result.test_result.status {
                                    TestStatus::None => {}
                                    TestStatus::PASS => {
                                        operation.add_success_toast("Test success.");
                                    }
                                    TestStatus::FAIL => {
                                        operation.add_error_toast("Test failed.");
                                    }
                                    TestStatus::WAIT => {}
                                    TestStatus::SKIP => {}
                                }
                            }
                        }

                    }
                    Err(e) => {
                        crt.record.must_get_mut_rest().error();
                        operation.add_error_toast(format!(
                            "Send request failed: {}",
                            e.error.to_string()
                        ));
                    }
                });
                self.send_promise = None;
            } else {
                ui.ctx().request_repaint();
                workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                    crt.record.must_get_mut_rest().pending()
                });
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
        egui::scroll_area::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for x in RequestPanelEnum::iter() {
                    ui.selectable_value(
                        &mut self.open_request_panel_enum,
                        x.clone(),
                        utils::build_with_count_ui_header(
                            x.to_string(),
                            RestPanel::get_count(crt.record.must_get_rest(), x, &parent_auth),
                            ui,
                        ),
                    );
                }
            });
        });
    }
}
