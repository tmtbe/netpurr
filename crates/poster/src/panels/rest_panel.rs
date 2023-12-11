use std::time::Instant;

use egui::{Button, Ui};
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{AppData, Header, Method, Request, Response};
use crate::panels::request_body_panel::RequestBodyPanel;
use crate::panels::request_headers_panel::RequestHeadersPanel;
use crate::panels::request_params_panel::RequestParamsPanel;
use crate::panels::response_panel::ResponsePanel;
use crate::panels::{DataView, HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct RestPanel {
    open_request_panel_enum: RequestPanelEnum,
    request_params_panel: RequestParamsPanel,
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
}

impl DataView for RestPanel {
    type CursorType = String;
    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        {
            let data = app_data
                .central_request_data_list
                .data_map
                .get_mut(cursor.as_str())
                .unwrap();
            data.rest.sync();
            ui.vertical(|ui| {
                if data.rest.request.base_url == "" {
                    ui.strong("Untitled Request");
                } else {
                    ui.strong(data.rest.request.base_url.clone());
                }
                ui.separator();
                ui.add_space(VERTICAL_GAP);
                ui.horizontal(|ui| {
                    egui::SidePanel::right("editor_right_panel")
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(HORIZONTAL_GAP);
                                if self.send_promise.is_some() {
                                    ui.add_enabled(false, Button::new("Send"));
                                } else {
                                    if ui.button("Send").clicked() {
                                        let send_response = app_data.rest_sender.send(
                                            &mut data.rest,
                                            app_data.environment.get_variable_hash_map(),
                                        );
                                        self.send_promise = Some(send_response.0);
                                        self.send_instant = Some(send_response.1);
                                        app_data.history_data_list.record(data.rest.clone())
                                    }
                                }
                                if ui.button("Save").clicked() {}
                            });
                        });
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
                                ui.centered_and_justified(|ui| {
                                    utils::highlight_template(
                                        ui,
                                        &mut data.rest.request.base_url,
                                        app_data.environment.get_variable_hash_map(),
                                    );
                                });
                            });
                        });
                });
                ui.add_space(HORIZONTAL_GAP);
                ui.separator();
                ui.add_space(HORIZONTAL_GAP);
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
                ui.separator();
                ui.add_space(HORIZONTAL_GAP);
            });

            if let Some(promise) = &self.send_promise {
                if let Some(result) = promise.ready() {
                    data.rest.elapsed_time = Some(self.send_instant.unwrap().elapsed().as_millis());
                    match result {
                        Ok(r) => {
                            data.rest.response = Response {
                                body: r.bytes.clone(),
                                headers: Header::new_from_tuple(r.headers.clone()),
                                url: r.url.clone(),
                                ok: r.ok.clone(),
                                status: r.status.clone(),
                                status_text: r.status_text.clone(),
                            };
                            data.rest.ready()
                        }
                        Err(_) => data.rest.error(),
                    }
                    self.send_promise = None;
                } else {
                    data.rest.pending()
                }
            }
        }
        match self.open_request_panel_enum {
            RequestPanelEnum::Params => {
                self.request_params_panel
                    .set_and_render(app_data, cursor.clone(), ui)
            }
            RequestPanelEnum::Authorization => {}
            RequestPanelEnum::Headers => {
                self.request_headers_panel
                    .set_and_render(app_data, cursor.clone(), ui)
            }
            RequestPanelEnum::Body => {
                self.request_body_panel
                    .set_and_render(app_data, cursor.clone(), ui)
            }
        }
        ui.separator();
        self.response_panel
            .set_and_render(app_data, cursor.clone(), ui)
    }
}
