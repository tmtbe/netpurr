use egui::Ui;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{AppData, Response};
use crate::panels::response_body_panel::ResponseBodyPanel;
use crate::panels::response_cookies_panel::ResponseCookiesPanel;
use crate::panels::response_headers_panel::ResponseHeadersPanel;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct ResponsePanel {
    status: ResponseStatus,
    open_panel_enum: ResponsePanelEnum,
    response_body_panel: ResponseBodyPanel,
    response_headers_panel: ResponseHeadersPanel,
    response_cookies_panel: ResponseCookiesPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum ResponsePanelEnum {
    Body,
    Cookies,
    Headers,
}

impl Default for ResponsePanelEnum {
    fn default() -> Self {
        ResponsePanelEnum::Body
    }
}

#[derive(PartialEq, Eq)]
enum ResponseStatus {
    None,
    Pending,
    Ready,
    Error,
}

impl Default for ResponseStatus {
    fn default() -> Self {
        ResponseStatus::None
    }
}

impl ResponsePanel {
    pub(crate) fn pending(&mut self) {
        self.status = ResponseStatus::Pending;
    }
    pub(crate) fn ready(&mut self) {
        self.status = ResponseStatus::Ready;
    }
    pub(crate) fn none(&mut self) {
        self.status = ResponseStatus::None;
    }
    pub(crate) fn error(&mut self) {
        self.status = ResponseStatus::Error;
    }
    fn get_count(response: &Response, panel_enum: ResponsePanelEnum) -> usize {
        match panel_enum {
            ResponsePanelEnum::Body => 0,
            ResponsePanelEnum::Cookies => response
                .headers
                .iter()
                .filter(|a| a.key.starts_with("set-cookie"))
                .count(),
            ResponsePanelEnum::Headers => response.headers.iter().count(),
        }
    }
}

impl DataView for ResponsePanel {
    type CursorType = String;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let data = app_data
            .central_request_data_list
            .data_map
            .get(cursor.as_str())
            .unwrap();
        match self.status {
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
                ui.horizontal(|ui| {
                    for x in ResponsePanelEnum::iter() {
                        ui.selectable_value(
                            &mut self.open_panel_enum,
                            x.clone(),
                            utils::build_with_count_ui_header(
                                x.to_string(),
                                ResponsePanel::get_count(&data.rest.response, x),
                                ui,
                            ),
                        );
                    }
                });
                ui.separator();
                match self.open_panel_enum {
                    ResponsePanelEnum::Body => {
                        self.response_body_panel
                            .set_and_render(app_data, cursor, ui);
                    }
                    ResponsePanelEnum::Cookies => {
                        self.response_cookies_panel
                            .set_and_render(app_data, cursor, ui);
                    }
                    ResponsePanelEnum::Headers => {
                        self.response_headers_panel
                            .set_and_render(app_data, cursor, ui);
                    }
                }
            }
            ResponseStatus::Error => {
                ui.centered_and_justified(|ui| {
                    ui.label("Could not get any response");
                });
            }
        }
    }
}
