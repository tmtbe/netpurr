use egui::Ui;
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{AppData, Method, Response};
use crate::panels::params_panel::ParamsPanel;
use crate::panels::reponse_panel::ResponsePanel;
use crate::panels::{DataView, HORIZONTAL_GAP, VERTICAL_GAP};

#[derive(Default)]
pub struct EditorPanel {
    open_request_panel_enum: RequestPanelEnum,
    params_panel: ParamsPanel,
    response_panel: ResponsePanel,
    send_promise: Option<Promise<ehttp::Result<ehttp::Response>>>,
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

impl DataView for EditorPanel {
    type CursorType = String;
    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        {
            let data = app_data
                .central_request_data_list
                .data_map
                .get_mut(cursor.as_str())
                .unwrap();
            ui.vertical(|ui| {
                ui.label(data.rest.request.url.clone());
                ui.separator();
                ui.add_space(VERTICAL_GAP);
                ui.horizontal(|ui| {
                    egui::SidePanel::right("editor_right_panel")
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                if self.send_promise.is_some() {
                                    ui.button("send").enabled = false
                                } else {
                                    if ui.button("Send").clicked() {
                                        self.send_promise =
                                            Some(app_data.rest_sender.send(&mut data.rest));
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
                                    ui.text_edit_singleline(&mut data.rest.request.url)
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
                            x.to_string(),
                        );
                    }
                });
                ui.separator();
                ui.add_space(HORIZONTAL_GAP);
            });
        }
        if let Some(promise) = &self.send_promise {
            if let Some(result) = promise.ready() {
                let id = app_data
                    .central_request_data_list
                    .select_id
                    .clone()
                    .unwrap();
                match result {
                    Ok(r) => {
                        app_data
                            .central_request_data_list
                            .data_map
                            .get_mut(id.as_str())
                            .unwrap()
                            .rest
                            .response = Response {
                            body: r.bytes.clone(),
                        };
                        self.response_panel.ready()
                    }
                    Err(_) => self.response_panel.error(),
                }
                self.send_promise = None;
            } else {
                self.response_panel.pending()
            }
        }
        match self.open_request_panel_enum {
            RequestPanelEnum::Params => {
                self.params_panel
                    .set_and_render(app_data, cursor.clone(), ui)
            }
            RequestPanelEnum::Authorization => {}
            RequestPanelEnum::Headers => {}
            RequestPanelEnum::Body => {}
        }
        ui.separator();
        self.response_panel
            .set_and_render(app_data, cursor.clone(), ui)
    }
}
