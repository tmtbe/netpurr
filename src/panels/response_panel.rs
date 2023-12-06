use egui::Ui;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::AppData;
use crate::panels::response_body_panel::ResponseBodyPanel;
use crate::panels::response_headers_panel::ResponseHeadersPanel;
use crate::panels::DataView;

#[derive(Default)]
pub struct ResponsePanel {
    status: ResponseStatus,
    open_panel_enum: PanelEnum,
    response_body_panel: ResponseBodyPanel,
    response_headers_panel: ResponseHeadersPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum PanelEnum {
    Body,
    Cookies,
    Headers,
}

impl Default for PanelEnum {
    fn default() -> Self {
        PanelEnum::Body
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
}

impl DataView for ResponsePanel {
    type CursorType = String;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
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
                    for x in PanelEnum::iter() {
                        ui.selectable_value(&mut self.open_panel_enum, x.clone(), x.to_string());
                    }
                });
                ui.separator();
                match self.open_panel_enum {
                    PanelEnum::Body => {
                        self.response_body_panel
                            .set_and_render(app_data, cursor, ui);
                    }
                    PanelEnum::Cookies => {}
                    PanelEnum::Headers => {
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
