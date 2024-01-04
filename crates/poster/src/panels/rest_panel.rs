use std::rc::Rc;

use egui::ahash::HashSet;
use egui::{Button, Label, RichText, Ui, Widget};
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{
    Auth, AuthType, BodyType, Header, HttpBody, HttpRecord, Method, Response, WorkspaceData,
};
use crate::operation::Operation;
use crate::panels::auth_panel::AuthPanel;
use crate::panels::request_body_panel::RequestBodyPanel;
use crate::panels::request_headers_panel::RequestHeadersPanel;
use crate::panels::request_params_panel::RequestParamsPanel;
use crate::panels::request_pre_script_panel::RequestPreScriptPanel;
use crate::panels::response_panel::ResponsePanel;
use crate::panels::{AlongDataView, DataView, HORIZONTAL_GAP};
use crate::script::script::{Context, Logger};
use crate::utils;
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct RestPanel {
    open_request_panel_enum: RequestPanelEnum,
    request_params_panel: RequestParamsPanel,
    auth_panel: AuthPanel,
    request_headers_panel: RequestHeadersPanel,
    request_body_panel: RequestBodyPanel,
    response_panel: ResponsePanel,
    request_pre_script_panel: RequestPreScriptPanel,
    send_promise: Option<Promise<ehttp::Result<ehttp::Response>>>,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum RequestPanelEnum {
    Params,
    Authorization,
    Headers,
    Body,
    PreRequestScript,
}

impl Default for RequestPanelEnum {
    fn default() -> Self {
        RequestPanelEnum::Params
    }
}

impl RestPanel {
    fn get_count(hr: &HttpRecord, panel_enum: RequestPanelEnum, auth: &Auth) -> usize {
        match panel_enum {
            RequestPanelEnum::Params => hr.request.params.iter().filter(|i| i.enable).count(),
            RequestPanelEnum::Authorization => match auth.get_final_type(auth.clone()) {
                AuthType::InheritAuthFromParent => 0,
                AuthType::NoAuth => 0,
                AuthType::BearerToken => usize::MAX,
                AuthType::BasicAuth => usize::MAX,
            },
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
        }
    }

    fn render_name_label(
        &mut self,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
    ) {
        let cookies_manager = workspace_data.cookies_manager.clone();
        let (mut data, envs, auth) = workspace_data.get_mut_crt_and_envs_auth(cursor.clone());
        data.rest.sync(envs.clone(), auth.clone(), cookies_manager);
        if data
            .rest
            .request
            .base_url
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            == ""
        {
            ui.strong("Untitled Request");
        } else {
            match &data.collection_path {
                None => {
                    ui.horizontal(|ui| {
                        ui.strong(data.rest.request.base_url.clone());
                    });
                }
                Some(collection_path) => {
                    ui.horizontal(|ui| {
                        Label::new(
                            RichText::new(collection_path)
                                .strong()
                                .background_color(ui.visuals().code_bg_color),
                        )
                        .ui(ui);
                        ui.strong(data.rest.request.base_url.as_str());
                    });
                }
            }
        }
    }

    fn render_editor_right_panel(
        &mut self,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
    ) {
        let mut send_rest = None;
        let (data, envs, auth) = workspace_data.get_mut_crt_and_envs_auth(cursor.clone());
        egui::SidePanel::right("editor_right_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP);
                    if self.send_promise.is_some() {
                        ui.add_enabled(false, Button::new("Send"));
                    } else {
                        if ui.button("Send").clicked() {
                            data.rest.request.clear_lock_with_script();
                            let mut send_envs = envs.clone();
                            if data.rest.pre_request_script != "" {
                                let js = data.rest.pre_request_script.clone();
                                let context = Context {
                                    request: data.rest.request.clone(),
                                    envs,
                                    logger: Logger::default(),
                                };
                                let result =
                                    operation.script_runtime().run(js, context).block_and_take();
                                if let Ok(new_context) = result {
                                    send_envs = new_context.envs;
                                    data.rest.request = new_context.request;
                                }
                            }
                            let send_response =
                                operation.rest_sender().send(&mut data.rest, send_envs);
                            self.send_promise = Some(send_response);
                            send_rest = Some(data.rest.clone());
                        }
                    }
                    if ui.button("Save").clicked() {
                        match &data.collection_path {
                            None => {
                                operation.open_windows().open_save(data.rest.clone(), None);
                            }
                            Some(collection_path) => {
                                operation
                                    .open_windows()
                                    .open_edit(data.rest.clone(), collection_path.clone());
                            }
                        }
                    }
                });
            });
        send_rest.map(|r| {
            workspace_data.history_data_list.record(r);
        });
    }

    fn render_editor_left_panel(
        &self,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
    ) {
        let (data, envs, auth) = workspace_data.get_mut_crt_and_envs_auth(cursor.clone());
        egui::SidePanel::left("editor_left_panel")
            .min_width(ui.available_width() - HORIZONTAL_GAP)
            .show_separator_line(false)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("method")
                        .selected_text(data.rest.request.method.clone().to_string())
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            for x in Method::iter() {
                                ui.selectable_value(
                                    &mut data.rest.request.method,
                                    x.clone(),
                                    x.to_string(),
                                );
                            }
                        });
                    let mut filter: HashSet<String> = HashSet::default();
                    filter.insert("?".to_string());
                    filter.insert(" ".to_string());
                    filter.insert("&".to_string());
                    ui.centered_and_justified(|ui| {
                        HighlightTemplateSinglelineBuilder::default()
                            .filter(filter)
                            .envs(envs.clone())
                            .all_space(false)
                            .build(cursor.clone() + "url", &mut data.rest.request.base_url)
                            .ui(ui);
                    });
                });
            });
    }

    fn render_request_open_panel(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: String,
    ) {
        let (data, envs, auth) = workspace_data.get_crt_and_envs_auth(cursor.clone());
        match self.open_request_panel_enum {
            RequestPanelEnum::Params => self.request_params_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                cursor.clone(),
            ),
            RequestPanelEnum::Authorization => {
                let mut parent_auth = None;
                match &data.collection_path {
                    None => {}
                    Some(collection_path) => {
                        parent_auth =
                            Some(workspace_data.collections.get_auth(collection_path.clone()));
                    }
                }
                self.auth_panel.set_envs(envs.clone(), parent_auth);
                {
                    let (data, envs, auth) =
                        workspace_data.get_mut_crt_and_envs_auth(cursor.clone());
                    self.auth_panel
                        .set_and_render(ui, &mut data.rest.request.auth);
                }
            }
            RequestPanelEnum::Headers => self.request_headers_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                cursor.clone(),
            ),
            RequestPanelEnum::Body => self.request_body_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                cursor.clone(),
            ),
            RequestPanelEnum::PreRequestScript => self.request_pre_script_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                cursor.clone(),
            ),
        }
    }

    fn send_promise(&mut self, ui: &mut Ui, workspace_data: &mut WorkspaceData, cursor: String) {
        let mut option_response_cookies = None;
        if let Some(promise) = &self.send_promise {
            let (data, envs, auth) = workspace_data.get_mut_crt_and_envs_auth(cursor.clone());
            if let Some(result) = promise.ready() {
                match result {
                    Ok(r) => {
                        data.rest.elapsed_time = Some(r.elapsed_time.as_millis());
                        data.rest.response = Response {
                            body: Rc::new(HttpBody::new(r.bytes.clone())),
                            headers: Header::new_from_tuple(r.headers.clone()),
                            url: r.url.clone(),
                            ok: r.ok.clone(),
                            status: r.status.clone(),
                            status_text: r.status_text.clone(),
                        };
                        option_response_cookies = Some(data.rest.response.get_cookies());
                        data.rest.ready()
                    }
                    Err(_) => data.rest.error(),
                }
                self.send_promise = None;
            } else {
                ui.ctx().request_repaint();
                data.rest.pending()
            }
        }
        option_response_cookies.map(|cookies| {
            for (_, c) in cookies.iter() {
                workspace_data.cookies_manager.add_domain_cookies(
                    c.domain.clone(),
                    c.name.clone(),
                    c.clone(),
                );
            }
        });
    }

    fn render_middle_select(
        &mut self,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
    ) {
        let (mut data, envs, auth) = workspace_data.get_crt_and_envs_auth(cursor.clone());
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
                                RestPanel::get_count(&data.rest, x, &auth),
                                ui,
                            ),
                        );
                    }
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    if ui.link("Cookies").clicked() {
                        operation.open_windows().open_cookies();
                    };
                    ui.link("Code");
                });
            },
        );
    }
}

impl DataView for RestPanel {
    type CursorType = String;
    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(HORIZONTAL_GAP);
                self.render_name_label(workspace_data, cursor.clone(), ui);
            });
            ui.separator();
            ui.horizontal(|ui| {
                self.render_editor_right_panel(operation, workspace_data, cursor.clone(), ui);
                self.render_editor_left_panel(workspace_data, cursor.clone(), ui);
            });
            ui.separator();
            self.render_middle_select(operation, workspace_data, cursor.clone(), ui);
            ui.separator();
        });
        self.send_promise(ui, workspace_data, cursor.clone());
        self.render_request_open_panel(ui, operation, workspace_data, cursor.clone());
        ui.separator();
        self.response_panel
            .set_and_render(ui, operation, workspace_data, cursor.clone());
    }
}
