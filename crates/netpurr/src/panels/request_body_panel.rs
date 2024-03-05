use egui::{Ui, Widget};
use serde_json::Value;
use strum::IntoEnumIterator;
use uuid::Uuid;

use netpurr_core::data::http::{BodyRawType, BodyType};

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::request_body_form_data_panel::RequestBodyFormDataPanel;
use crate::panels::request_body_xxx_form_panel::RequestBodyXXXFormPanel;
use crate::panels::{HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;
use crate::utils::openapi_help::OpenApiHelp;
use crate::utils::{openapi_help, HighlightValue};
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;
use crate::windows::view_json_windows::ViewJsonWindows;

#[derive(Default)]
pub struct RequestBodyPanel {
    request_body_form_data_panel: RequestBodyFormDataPanel,
    request_body_xxx_form_panel: RequestBodyXXXFormPanel,
}

impl RequestBodyPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
        ui.horizontal_wrapped(|ui| {
            ui.add_space(HORIZONTAL_GAP);
            for x in BodyType::iter() {
                crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                    utils::selectable_check(
                        ui,
                        &mut crt.record.must_get_mut_rest().request.body.body_type,
                        x.clone(),
                        x.to_string(),
                    );
                });
            }
        });
        ui.horizontal(|ui| {
            if crt.record.must_get_rest().request.body.body_type == BodyType::RAW {
                egui::ComboBox::from_id_source("body_raw_type")
                    .selected_text(
                        crt.record
                            .must_get_rest()
                            .request
                            .body
                            .body_raw_type
                            .clone()
                            .to_string(),
                    )
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for body_raw_type in BodyRawType::iter() {
                            crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                ui.selectable_value(
                                    &mut crt.record.must_get_mut_rest().request.body.body_raw_type,
                                    body_raw_type.clone(),
                                    body_raw_type.to_string(),
                                );
                            });
                        }
                    });
                if crt.record.must_get_rest().request.body.body_raw_type == BodyRawType::JSON {
                    if ui.button("Pretty").clicked() {
                        let json = crt.record.must_get_rest().request.body.body_str.clone();
                        let value = serde_json::from_str::<Value>(&json);
                        match value {
                            Ok(v) => {
                                let pretty_json = serde_json::to_string_pretty(&v);
                                match pretty_json {
                                    Ok(pretty_json_str) => {
                                        crt = workspace_data.must_get_mut_crt(
                                            crt_id.clone(),
                                            |crt| {
                                                crt.record
                                                    .must_get_mut_rest()
                                                    .request
                                                    .body
                                                    .body_str = pretty_json_str;
                                            },
                                        );
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
                if let Some(operation_id) = crt.record.must_get_rest().operation_id.clone() {
                    if ui.button("Generate Schema").clicked() {
                        if let Some(collection) =
                            workspace_data.get_collection(crt.collection_path.clone())
                        {
                            if let Some(openapi) = collection.openapi {
                                let openapi_help = OpenApiHelp { openapi };
                                let schema_value = openapi_help.gen_openapi_schema(operation_id);
                                match schema_value {
                                    None => {}
                                    Some(value) => {
                                        operation.add_success_toast("Generate schema success");
                                        operation.add_window(Box::new(
                                            ViewJsonWindows::default().with_json(
                                                serde_json::to_string(&value).unwrap(),
                                                Uuid::new_v4().to_string(),
                                            ),
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        ui.add_space(VERTICAL_GAP);
        ui.push_id("request_body_select_type", |ui| {
            match crt.record.must_get_rest().request.body.body_type {
                BodyType::NONE => {
                    ui.label("This request does not have a body");
                }
                BodyType::FROM_DATA => {
                    self.request_body_form_data_panel
                        .set_and_render(ui, workspace_data, crt_id)
                }
                BodyType::X_WWW_FROM_URLENCODED => {
                    self.request_body_xxx_form_panel
                        .set_and_render(ui, workspace_data, crt_id)
                }
                BodyType::RAW => {
                    ui.push_id("request_body", |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 25.0)
                            .show(ui, |ui| {
                                crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                    HighlightTemplateSinglelineBuilder::default()
                                        .multiline()
                                        .envs(envs)
                                        .all_space(true)
                                        .build(
                                            "request_body".to_string(),
                                            &mut crt
                                                .record
                                                .must_get_mut_rest()
                                                .request
                                                .body
                                                .body_str,
                                        )
                                        .ui(ui);
                                });
                            });
                    });
                }
                BodyType::BINARY => {
                    let mut button_name = utils::build_with_count_ui_header(
                        "Select File".to_string(),
                        HighlightValue::None,
                        ui,
                    );
                    if crt.record.must_get_rest().request.body.body_file != "" {
                        button_name = utils::build_with_count_ui_header(
                            "Select File".to_string(),
                            HighlightValue::Usize(1),
                            ui,
                        );
                    }
                    ui.horizontal(|ui| {
                        if ui.button(button_name).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                    crt.record.must_get_mut_rest().request.body.body_file =
                                        path.display().to_string();
                                });
                            }
                        }
                        let mut path = crt.record.must_get_rest().request.body.body_file.clone();
                        utils::text_edit_singleline_justify(ui, &mut path);
                    });
                }
            }
        });
    }
}
