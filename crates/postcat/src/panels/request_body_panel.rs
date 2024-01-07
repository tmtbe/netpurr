use egui::{Ui, Widget};
use strum::IntoEnumIterator;

use crate::data::http::{BodyRawType, BodyType};
use crate::data::workspace::WorkspaceData;
use crate::operation::Operation;
use crate::panels::request_body_form_data_panel::RequestBodyFormDataPanel;
use crate::panels::request_body_xxx_form_panel::RequestBodyXXXFormPanel;
use crate::panels::{DataView, HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;
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
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        let (data, envs, _) = workspace_data.get_mut_crt_and_envs_parent_auth(cursor.clone());
        ui.horizontal(|ui| {
            ui.add_space(HORIZONTAL_GAP);
            for x in BodyType::iter() {
                utils::selectable_check(
                    ui,
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
            BodyType::FROM_DATA => self.request_body_form_data_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                cursor,
            ),
            BodyType::X_WWW_FROM_URLENCODED => self.request_body_xxx_form_panel.set_and_render(
                ui,
                operation,
                workspace_data,
                cursor,
            ),
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
            BodyType::BINARY => {
                let mut button_name =
                    utils::build_with_count_ui_header("Select File".to_string(), 0, ui);
                if data.rest.request.body.body_file != "" {
                    button_name =
                        utils::build_with_count_ui_header("Select File".to_string(), 1, ui);
                }
                ui.horizontal(|ui| {
                    if ui.button(button_name).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            data.rest.request.body.body_file = path.display().to_string();
                        }
                    }
                    let mut path = data.rest.request.body.body_file.clone();
                    utils::text_edit_singleline_justify(ui, &mut path);
                });
            }
        }
    }
}
