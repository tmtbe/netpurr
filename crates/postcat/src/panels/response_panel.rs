use std::collections::BTreeMap;

use egui::{RichText, Ui};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::central_request_data::CentralRequestItem;
use crate::data::cookies_manager::Cookie;
use crate::data::http::{Response, ResponseStatus};
use crate::data::test::{TestResult, TestStatus};
use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::response_body_panel::ResponseBodyPanel;
use crate::panels::response_cookies_panel::ResponseCookiesPanel;
use crate::panels::response_headers_panel::ResponseHeadersPanel;
use crate::panels::response_log_panel::ResponseLogPanel;
use crate::panels::test_result_panel::TestResultPanel;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct ResponsePanel {
    open_panel_enum: ResponsePanelEnum,
    response_body_panel: ResponseBodyPanel,
    response_headers_panel: ResponseHeadersPanel,
    response_cookies_panel: ResponseCookiesPanel,
    response_log_panel: ResponseLogPanel,
    test_result_panel: TestResultPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum ResponsePanelEnum {
    Body,
    Cookies,
    Headers,
    Logs,
    TestResult,
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
        test_result: &TestResult,
        panel_enum: ResponsePanelEnum,
    ) -> usize {
        match panel_enum {
            ResponsePanelEnum::Body => 0,
            ResponsePanelEnum::Cookies => cookies.len(),
            ResponsePanelEnum::Headers => response.headers.iter().count(),
            ResponsePanelEnum::Logs => response.logger.logs.len(),
            ResponsePanelEnum::TestResult => match test_result.status {
                TestStatus::None => 0,
                TestStatus::PASS => usize::MAX,
                TestStatus::FAIL => usize::MAX,
            },
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
                    for response_panel_enum in ResponsePanelEnum::iter() {
                        ui.selectable_value(
                            &mut self.open_panel_enum,
                            response_panel_enum.clone(),
                            utils::build_with_count_ui_header(
                                response_panel_enum.to_string(),
                                ResponsePanel::get_count(
                                    &data.rest.response,
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
            ResponsePanelEnum::TestResult => {
                self.test_result_panel
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
        crt_id: Self::CursorType,
    ) {
        let crt = workspace_data.must_get_crt(crt_id.clone());
        let cookies = workspace_data.get_url_cookies(crt.rest.request.base_url.clone());
        match crt.rest.status {
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
                self.build_ready_panel(operation, workspace_data, crt_id, ui, &crt, cookies);
            }
            ResponseStatus::Error => {
                ui.centered_and_justified(|ui| {
                    ui.label("Could not get any response");
                });
            }
        }
    }
}
