use std::collections::BTreeMap;

use egui::{Color32, RichText, Ui};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use netpurr_core::data::cookies_manager::Cookie;
use netpurr_core::data::http::{Response, ResponseStatus};
use netpurr_core::data::test::{TestResult, TestStatus};

use crate::data::central_request_data::CentralRequestItem;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::response_body_panel::ResponseBodyPanel;
use crate::panels::response_cookies_panel::ResponseCookiesPanel;
use crate::panels::response_headers_panel::ResponseHeadersPanel;
use crate::panels::response_log_panel::ResponseLogPanel;
use crate::panels::test_result_panel::TestResultPanel;
use crate::utils;
use crate::utils::HighlightValue;

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
        match crt.record.must_get_rest().status {
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
    fn get_count(
        response: &Response,
        cookies: &BTreeMap<String, Cookie>,
        test_result: &TestResult,
        panel_enum: ResponsePanelEnum,
    ) -> HighlightValue {
        match panel_enum {
            ResponsePanelEnum::Body => HighlightValue::None,
            ResponsePanelEnum::Cookies => HighlightValue::Usize(cookies.len()),
            ResponsePanelEnum::Headers => HighlightValue::Usize(response.headers.iter().count()),
            ResponsePanelEnum::Logs => HighlightValue::Usize(response.logger.logs.len()),
            ResponsePanelEnum::TestResult => match test_result.status {
                TestStatus::None => HighlightValue::None,
                TestStatus::PASS => HighlightValue::String(
                    format!(
                        "{}/{}",
                        test_result.test_info_list.len(),
                        test_result.test_info_list.len()
                    ),
                    Color32::DARK_GREEN,
                ),
                TestStatus::FAIL => HighlightValue::String(
                    format!(
                        "{}/{}",
                        test_result
                            .test_info_list
                            .iter()
                            .filter(|i| i.status == TestStatus::PASS)
                            .count(),
                        test_result.test_info_list.len()
                    ),
                    Color32::RED,
                ),
            },
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
        ui.horizontal(|ui| {
            ui.link("Code");
        });
        ui.horizontal(|ui| {
            for response_panel_enum in ResponsePanelEnum::iter() {
                ui.selectable_value(
                    &mut self.open_panel_enum,
                    response_panel_enum.clone(),
                    utils::build_with_count_ui_header(
                        response_panel_enum.to_string(),
                        ResponsePanel::get_count(
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
        ui.separator();
        match self.open_panel_enum {
            ResponsePanelEnum::Body => {
                self.response_body_panel
                    .set_and_render(ui, workspace_data, operation, crt_id);
            }
            ResponsePanelEnum::Cookies => {
                self.response_cookies_panel.set_and_render(ui, &cookies);
            }
            ResponsePanelEnum::Headers => {
                self.response_headers_panel
                    .set_and_render(ui, workspace_data, crt_id);
            }
            ResponsePanelEnum::Logs => {
                self.response_log_panel
                    .set_and_render(ui, workspace_data, crt_id);
            }
            ResponsePanelEnum::TestResult => {
                self.test_result_panel
                    .set_and_render(ui, workspace_data, crt_id);
            }
        }
    }
}
