use egui::Ui;
use strum::IntoEnumIterator;

use crate::data::{AppData, BodyRawType, BodyType};
use crate::panels::request_body_form_data_panel::RequestBodyFormDataPanel;
use crate::panels::request_body_xxx_form_panel::RequestBodyXXXFormPanel;
use crate::panels::{DataView, VERTICAL_GAP};

#[derive(Default)]
pub struct RequestBodyPanel {
    request_body_form_data_panel: RequestBodyFormDataPanel,
    request_body_xxx_form_panel: RequestBodyXXXFormPanel,
}

impl DataView for RequestBodyPanel {
    type CursorType = String;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let (data, envs, auth) = app_data.get_mut_crt_and_envs_auth(cursor.clone());
        ui.horizontal(|ui| {
            for x in BodyType::iter() {
                ui.selectable_value(&mut data.rest.request.body_type, x.clone(), x.to_string());
            }
            if data.rest.request.body_type == BodyType::RAW {
                egui::ComboBox::from_id_source("body_raw_type")
                    .selected_text(data.rest.request.body_raw_type.clone().to_string())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for x in BodyRawType::iter() {
                            ui.selectable_value(
                                &mut data.rest.request.body_raw_type,
                                x.clone(),
                                x.to_string(),
                            );
                        }
                    });
            }
        });
        ui.add_space(VERTICAL_GAP);
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let layout_job =
                crate::widgets::highlight::highlight_template(string, 12.0, ui, envs.clone());
            ui.fonts(|f| f.layout_job(layout_job))
        };
        match data.rest.request.body_type {
            BodyType::NONE => {
                ui.label("This request does not have a body");
            }
            BodyType::FROM_DATA => self
                .request_body_form_data_panel
                .set_and_render(app_data, cursor, ui),
            BodyType::X_WWW_FROM_URLENCODED => self
                .request_body_xxx_form_panel
                .set_and_render(app_data, cursor, ui),
            BodyType::RAW => {
                ui.push_id("request_body", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut data.rest.request.body_str)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .desired_rows(10)
                                    .code_editor()
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut layouter),
                            );
                        });
                });
            }
            BodyType::BINARY => {}
        }
    }
}
