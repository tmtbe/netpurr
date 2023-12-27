use std::rc::Rc;
use std::time::Instant;

use egui::ahash::HashSet;
use egui::{Button, Label, RichText, Ui, Widget};
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{AppData, Header, HttpBody, Method, Request, Response};
use crate::panels::auth_panel::AuthPanel;
use crate::panels::request_body_panel::RequestBodyPanel;
use crate::panels::request_headers_panel::RequestHeadersPanel;
use crate::panels::request_params_panel::RequestParamsPanel;
use crate::panels::response_panel::ResponsePanel;
use crate::panels::{AlongDataView, DataView, HORIZONTAL_GAP};
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
    send_promise: Option<Promise<ehttp::Result<ehttp::Response>>>,
    send_instant: Option<Instant>,
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
    fn get_count(request: &Request, panel_enum: RequestPanelEnum) -> usize {
        match panel_enum {
            RequestPanelEnum::Params => request.params.iter().filter(|i| i.enable).count(),
            RequestPanelEnum::Authorization => 0,
            RequestPanelEnum::Headers => request.headers.iter().filter(|i| i.enable).count(),
            RequestPanelEnum::Body => 0,
        }
    }

    fn render_name_label(&mut self, app_data: &mut AppData, cursor: String, ui: &mut Ui) {
        let cookies_manager = app_data.rest_sender.cookies_manager.clone();
        let (mut data, envs, auth) = app_data.get_mut_crt_and_envs_auth(cursor.clone());
        data.rest.sync(envs.clone(), auth.clone(), cookies_manager);
        if data.rest.request.base_url == "" {
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

    fn render_editor_right_panel(&mut self, app_data: &mut AppData, cursor: String, ui: &mut Ui) {
        let (mut data, envs, auth) = app_data.get_crt_and_envs_auth(cursor.clone());
        egui::SidePanel::right("editor_right_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP);
                    if self.send_promise.is_some() {
                        ui.add_enabled(false, Button::new("Send"));
                    } else {
                        if ui.button("Send").clicked() {
                            let send_response =
                                app_data.rest_sender.send(&mut data.rest, envs.clone());
                            self.send_promise = Some(send_response.0);
                            self.send_instant = Some(send_response.1);
                            app_data.history_data_list.record(data.rest.clone());
                            app_data.central_request_data_list.refresh(data.clone());
                        }
                    }
                    if ui.button("Save").clicked() {
                        match data.collection_path {
                            None => {
                                app_data.open_windows().open_save(data.rest.clone(), None);
                            }
                            Some(collection_path) => {
                                app_data
                                    .open_windows()
                                    .open_edit(data.rest.clone(), collection_path);
                            }
                        }
                    }
                });
            });
    }

    fn render_editor_left_panel(&self, app_data: &mut AppData, cursor: String, ui: &mut Ui) {
        let (data, envs, auth) = app_data.get_mut_crt_and_envs_auth(cursor.clone());
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

    fn render_request_open_panel(&mut self, app_data: &mut AppData, cursor: String, ui: &mut Ui) {
        let (data, envs, auth) = app_data.get_crt_and_envs_auth(cursor.clone());
        match self.open_request_panel_enum {
            RequestPanelEnum::Params => {
                self.request_params_panel
                    .set_and_render(ui, app_data, cursor.clone())
            }
            RequestPanelEnum::Authorization => {
                let mut parent_auth = None;
                match &data.collection_path {
                    None => {}
                    Some(collection_path) => {
                        parent_auth = Some(app_data.collections.get_auth(collection_path.clone()));
                    }
                }
                self.auth_panel.set_envs(envs.clone(), parent_auth);
                {
                    let (data, envs, auth) = app_data.get_mut_crt_and_envs_auth(cursor.clone());
                    self.auth_panel
                        .set_and_render(ui, &mut data.rest.request.auth);
                }
            }
            RequestPanelEnum::Headers => {
                self.request_headers_panel
                    .set_and_render(ui, app_data, cursor.clone())
            }
            RequestPanelEnum::Body => {
                self.request_body_panel
                    .set_and_render(ui, app_data, cursor.clone())
            }
        }
    }

    fn send_promise(&mut self, app_data: &mut AppData, cursor: String) {
        if let Some(promise) = &self.send_promise {
            let (mut data, envs, auth) = app_data.get_crt_and_envs_auth(cursor.clone());
            if let Some(result) = promise.ready() {
                data.rest.elapsed_time = Some(self.send_instant.unwrap().elapsed().as_millis());
                match result {
                    Ok(r) => {
                        data.rest.response = Response {
                            body: Rc::new(HttpBody::new(r.bytes.clone())),
                            headers: Header::new_from_tuple(r.headers.clone()),
                            url: r.url.clone(),
                            ok: r.ok.clone(),
                            status: r.status.clone(),
                            status_text: r.status_text.clone(),
                        };
                        let cookies = data.rest.response.get_cookies();
                        for (_, c) in cookies.iter() {
                            app_data.rest_sender.cookies_manager.add_domain_cookies(
                                c.domain.clone(),
                                c.name.clone(),
                                c.clone(),
                            );
                        }
                        data.rest.ready()
                    }
                    Err(_) => data.rest.error(),
                }
                self.send_promise = None;
            } else {
                data.rest.pending()
            }
            app_data.central_request_data_list.refresh(data);
        }
    }

    fn render_middle_select(&mut self, app_data: &mut AppData, cursor: String, ui: &mut Ui) {
        let (mut data, envs, auth) = app_data.get_crt_and_envs_auth(cursor.clone());
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
                                RestPanel::get_count(&data.rest.request, x),
                                ui,
                            ),
                        );
                    }
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    if ui.link("Cookies").clicked() {
                        app_data.open_windows().open_cookies();
                    };
                    ui.link("Code");
                });
            },
        );
    }
}

impl DataView for RestPanel {
    type CursorType = String;
    fn set_and_render(&mut self, ui: &mut Ui, app_data: &mut AppData, cursor: Self::CursorType) {
        ui.vertical(|ui| {
            self.render_name_label(app_data, cursor.clone(), ui);
            ui.separator();
            ui.horizontal(|ui| {
                self.render_editor_right_panel(app_data, cursor.clone(), ui);
                self.render_editor_left_panel(app_data, cursor.clone(), ui);
            });
            ui.separator();
            self.render_middle_select(app_data, cursor.clone(), ui);
            ui.separator();
        });
        self.send_promise(app_data, cursor.clone());
        self.render_request_open_panel(app_data, cursor.clone(), ui);
        ui.separator();
        self.response_panel
            .set_and_render(ui, app_data, cursor.clone());
    }
}
