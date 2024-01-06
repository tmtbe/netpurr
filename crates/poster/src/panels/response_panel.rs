use std::collections::BTreeMap;

use egui::{RichText, Ui};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{CentralRequestItem, Cookie, Response, ResponseStatus, WorkspaceData};
use crate::operation::Operation;
use crate::panels::response_body_panel::ResponseBodyPanel;
use crate::panels::response_cookies_panel::ResponseCookiesPanel;
use crate::panels::response_headers_panel::ResponseHeadersPanel;
use crate::panels::response_log_panel::ResponseLogPanel;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct ResponsePanel {
    open_panel_enum: ResponsePanelEnum,
    response_body_panel: ResponseBodyPanel,
    response_headers_panel: ResponseHeadersPanel,
    response_cookies_panel: ResponseCookiesPanel,
    response_log_panel: ResponseLogPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum ResponsePanelEnum {
    Body,
    Cookies,
    Headers,
    Logs,
}

impl Default for ResponsePanelEnum {
    fn default() -> Self {
        ResponsePanelEnum::Body
    }
}

impl ResponsePanel {
    fn get_count(
        response: &Response,
        cookies: &BTreeMap<String, Cookie>,
        panel_enum: ResponsePanelEnum,
    ) -> usize {
        match panel_enum {
            ResponsePanelEnum::Body => 0,
            ResponsePanelEnum::Cookies => cookies.len(),
            ResponsePanelEnum::Headers => response.headers.iter().count(),
            ResponsePanelEnum::Logs => response.logger.logs.len(),
        }
    }

    fn build_ready_panel(
        &mut self,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: String,
        ui: &mut Ui,
        data: &CentralRequestItem,
        cookies: BTreeMap<String, Cookie>,
    ) {
        utils::left_right_panel(
            ui,
            "response".to_string(),
            |ui| {
                ui.horizontal(|ui| {
                    for x in ResponsePanelEnum::iter() {
                        ui.selectable_value(
                            &mut self.open_panel_enum,
                            x.clone(),
                            utils::build_with_count_ui_header(
                                x.to_string(),
                                ResponsePanel::get_count(&data.rest.response, &cookies, x),
                                ui,
                            ),
                        );
                    }
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label(
                        RichText::new(data.rest.response.status.to_string())
                            .color(ui.visuals().warn_fg_color)
                            .strong(),
                    );

                    ui.label("Time:");
                    ui.label(
                        RichText::new(data.rest.response.elapsed_time.to_string() + "ms")
                            .color(ui.visuals().warn_fg_color)
                            .strong(),
                    );

                    ui.label("Size:");
                    ui.label(
                        RichText::new(data.rest.response.body.get_byte_size())
                            .color(ui.visuals().warn_fg_color)
                            .strong(),
                    );
                });
            },
        );
        ui.separator();
        match self.open_panel_enum {
            ResponsePanelEnum::Body => {
                self.response_body_panel
                    .set_and_render(ui, operation, workspace_data, cursor);
            }
            ResponsePanelEnum::Cookies => {
                self.response_cookies_panel.set_and_render(ui, &cookies);
            }
            ResponsePanelEnum::Headers => {
                self.response_headers_panel
                    .set_and_render(ui, operation, workspace_data, cursor);
            }
            ResponsePanelEnum::Logs => {
                self.response_log_panel
                    .set_and_render(ui, operation, workspace_data, cursor);
            }
        }
    }
}

impl DataView for ResponsePanel {
    type CursorType = String;

    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        let data = workspace_data
            .central_request_data_list
            .data_map
            .get(cursor.as_str())
            .cloned()
            .unwrap();
        let cookies = workspace_data
            .cookies_manager
            .get_url_cookies(data.rest.request.base_url.clone());
        match data.rest.status {
            ResponseStatus::None => {
                ui.strong("Response");
                ui.separator();
                ui.centered_and_justified(|ui| {
                    ui.label("Hit the Send button to get a response");
                });
            }
            ResponseStatus::Pending => {
                ui.centered_and_justified(|ui| {
                    ui.label("Loading...");
                });
            }

            ResponseStatus::Ready => {
                self.build_ready_panel(operation, workspace_data, cursor, ui, &data, cookies);
            }
            ResponseStatus::Error => {
                ui.centered_and_justified(|ui| {
                    ui.label("Could not get any response");
                });
            }
        }
    }
}
