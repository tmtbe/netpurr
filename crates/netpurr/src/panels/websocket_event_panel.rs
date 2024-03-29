use std::collections::BTreeMap;

use chrono::format::StrftimeItems;
use eframe::emath::Align;
use egui::{Layout, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use netpurr_core::data::central_request_data::CentralRequestItem;
use netpurr_core::data::cookies_manager::Cookie;
use netpurr_core::data::http::Response;
use netpurr_core::data::test::TestResult;
use netpurr_core::data::websocket::{WebSocketMessage, WebSocketStatus};
use netpurr_core::data::workspace_data::WorkspaceData;

use crate::operation::operation::Operation;
use crate::panels::response_cookies_panel::ResponseCookiesPanel;
use crate::panels::response_headers_panel::ResponseHeadersPanel;
use crate::panels::response_log_panel::ResponseLogPanel;
use crate::utils;
use crate::utils::HighlightValue;

#[derive(Default)]
pub struct WebsocketEventPanel {
    open_panel_enum: ResponsePanelEnum,
    response_headers_panel: ResponseHeadersPanel,
    response_cookies_panel: ResponseCookiesPanel,
    response_log_panel: ResponseLogPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum ResponsePanelEnum {
    Event,
    Cookies,
    Headers,
    Logs,
}

impl Default for ResponsePanelEnum {
    fn default() -> Self {
        ResponsePanelEnum::Event
    }
}

impl WebsocketEventPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let crt = workspace_data.must_get_crt(crt_id.clone());
        let cookies = workspace_data
            .get_url_cookies(crt.record.must_get_rest().request.get_url_with_schema());
        match &crt.record.must_get_websocket().session {
            None => {
                ui.strong("Response");
                ui.separator();
                ui.centered_and_justified(|ui| {
                    ui.label("Hit the Connect button to connect websocket server");
                });
            }
            Some(session) => match session.get_status() {
                WebSocketStatus::Connect => {
                    self.build_ready_panel(operation, workspace_data, crt_id, ui, &crt, cookies);
                }
                WebSocketStatus::Connecting => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Connecting...");
                    });
                }
                WebSocketStatus::Disconnect => {}
                WebSocketStatus::ConnectError(e) => {
                    ui.centered_and_justified(|ui| {
                        ui.label(e);
                    });
                }
                WebSocketStatus::SendError(_) => {}
                WebSocketStatus::SendSuccess => {}
            },
        }
    }
    fn get_count(
        response: &Response,
        cookies: &BTreeMap<String, Cookie>,
        test_result: &TestResult,
        panel_enum: ResponsePanelEnum,
    ) -> HighlightValue {
        match panel_enum {
            ResponsePanelEnum::Event => HighlightValue::None,
            ResponsePanelEnum::Cookies => HighlightValue::Usize(cookies.len()),
            ResponsePanelEnum::Headers => HighlightValue::Usize(response.headers.iter().count()),
            ResponsePanelEnum::Logs => HighlightValue::Usize(response.logger.logs.len()),
        }
    }

    fn build_ready_panel(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
        ui: &mut Ui,
        data: &CentralRequestItem,
        cookies: BTreeMap<String, Cookie>,
    ) {
        utils::left_right_panel(
            ui,
            "response".to_string(),
            |ui| {
                ui.horizontal(|ui| {
                    for response_panel_enum in ResponsePanelEnum::iter() {
                        ui.selectable_value(
                            &mut self.open_panel_enum,
                            response_panel_enum.clone(),
                            utils::build_with_count_ui_header(
                                response_panel_enum.to_string(),
                                WebsocketEventPanel::get_count(
                                    &data.record.must_get_rest().response,
                                    &cookies,
                                    &data.test_result,
                                    response_panel_enum,
                                ),
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
                        RichText::new(data.record.must_get_rest().response.status.to_string())
                            .color(ui.visuals().warn_fg_color)
                            .strong(),
                    );

                    ui.label("Time:");
                    ui.label(
                        RichText::new(
                            data.record
                                .must_get_rest()
                                .response
                                .elapsed_time
                                .to_string()
                                + "ms",
                        )
                        .color(ui.visuals().warn_fg_color)
                        .strong(),
                    );

                    ui.label("Size:");
                    ui.label(
                        RichText::new(data.record.must_get_rest().response.body.get_byte_size())
                            .color(ui.visuals().warn_fg_color)
                            .strong(),
                    );
                });
            },
        );
        ui.separator();
        let crt = workspace_data.must_get_crt(crt_id.clone());
        match self.open_panel_enum {
            ResponsePanelEnum::Event => {
                self.render_event(ui, workspace_data, operation, crt_id);
            }
            ResponsePanelEnum::Cookies => {
                self.response_cookies_panel.set_and_render(ui, &cookies);
            }
            ResponsePanelEnum::Headers => {
                self.response_headers_panel
                    .set_and_render(ui, &crt.record.must_get_rest().response);
            }
            ResponsePanelEnum::Logs => {
                self.response_log_panel
                    .set_and_render(ui, &crt.record.must_get_rest().response);
            }
        }
    }
    fn render_event(
        &self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        crt_id: String,
    ) {
        match &workspace_data
            .must_get_crt(crt_id)
            .record
            .must_get_websocket()
            .session
        {
            None => {}
            Some(session) => {
                let available_width = ui.available_width();
                let table = TableBuilder::new(ui)
                    .resizable(false)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(Column::exact(50.0))
                    .column(Column::initial(available_width - 200.0).range(300.0..=1000.0))
                    .column(Column::remainder())
                    .max_scroll_height(100.0);
                table
                    .striped(true)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("");
                        });
                        header.col(|ui| {
                            ui.strong("Data");
                        });
                        header.col(|ui| {
                            ui.strong("Time");
                        });
                    })
                    .body(|mut body| {
                        for (index, message) in session.get_messages().iter().rev().enumerate() {
                            let mut flag = "Send";
                            let mut text = "".to_string();
                            let mut time = "".to_string();
                            match message {
                                WebSocketMessage::Send(d, msg_type, msg) => {
                                    flag = "Send";
                                    text = msg.to_string();
                                    time = d
                                        .format_with_items(StrftimeItems::new("%H:%M:%S"))
                                        .to_string();
                                }
                                WebSocketMessage::Receive(d, msg_type, msg) => {
                                    flag = "Receive";
                                    text = msg.to_string();
                                    time = d
                                        .format_with_items(StrftimeItems::new("%H:%M:%S"))
                                        .to_string();
                                }
                            }
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(flag);
                                });
                                row.col(|ui| {
                                    ui.label(text.replace("\n", ""));
                                });
                                row.col(|ui| {
                                    ui.label(time);
                                });
                            });
                        }
                    });
            }
        }
    }
}
