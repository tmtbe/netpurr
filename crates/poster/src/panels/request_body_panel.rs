use egui::{Ui, Widget};
use strum::IntoEnumIterator;

use crate::data::{AppData, BodyRawType, BodyType};
use crate::operation::Operation;
use crate::panels::request_body_form_data_panel::RequestBodyFormDataPanel;
use crate::panels::request_body_xxx_form_panel::RequestBodyXXXFormPanel;
use crate::panels::{DataView, VERTICAL_GAP};
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct RequestBodyPanel {
    request_body_form_data_panel: RequestBodyFormDataPanel,
    request_body_xxx_form_panel: RequestBodyXXXFormPanel,
}

impl DataView for RequestBodyPanel {
    type CursorType = String;

    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        app_data: &mut AppData,
        cursor: Self::CursorType,
    ) {
        let (data, envs, auth) = app_data.get_mut_crt_and_envs_auth(cursor.clone());
        ui.horizontal(|ui| {
            for x in BodyType::iter() {
                ui.selectable_value(
                    &mut data.rest.request.body.body_type,
                    x.clone(),
                    x.to_string(),
                );
            }
            if data.rest.request.body.body_type == BodyType::RAW {
                egui::ComboBox::from_id_source("body_raw_type")
                    .selected_text(data.rest.request.body.body_raw_type.clone().to_string())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for x in BodyRawType::iter() {
                            ui.selectable_value(
                                &mut data.rest.request.body.body_raw_type,
                                x.clone(),
                                x.to_string(),
                            );
                        }
                    });
            }
        });
        ui.add_space(VERTICAL_GAP);
        match data.rest.request.body.body_type {
            BodyType::NONE => {
                ui.label("This request does not have a body");
            }
            BodyType::FROM_DATA => self
                .request_body_form_data_panel
                .set_and_render(ui, operation, app_data, cursor),
            BodyType::X_WWW_FROM_URLENCODED => self
                .request_body_xxx_form_panel
                .set_and_render(ui, operation, app_data, cursor),
            BodyType::RAW => {
                ui.push_id("request_body", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            HighlightTemplateSinglelineBuilder::default()
                                .multiline()
                                .envs(envs)
                                .all_space(true)
                                .build(
                                    "request_body".to_string(),
                                    &mut data.rest.request.body.body_str,
                                )
                                .ui(ui);
                        });
                });
            }
            BodyType::BINARY => {}
        }
    }
}
