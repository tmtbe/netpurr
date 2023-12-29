use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextBuffer, TextEdit, Widget};
use egui_extras::{Column, TableBody, TableBuilder, TableRow};
use strum::IntoEnumIterator;

use crate::data::{CentralRequestItem, MultipartData, MultipartDataType, WorkspaceData};
use crate::operation::Operation;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct RequestBodyFormDataPanel {
    new_form: MultipartData,
}

impl DataView for RequestBodyFormDataPanel {
    type CursorType = String;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        let data = workspace_data
            .central_request_data_list
            .data_map
            .get_mut(cursor.as_str())
            .unwrap();
        let mut delete_index = None;
        let table = TableBuilder::new(ui)
            .resizable(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::exact(20.0))
            .column(Column::exact(100.0))
            .column(Column::initial(200.0).range(40.0..=300.0))
            .column(Column::initial(200.0).range(40.0..=300.0))
            .column(Column::remainder())
            .max_scroll_height(100.0);
        table.header(20.0, self.build_header()).body(|mut body| {
            delete_index = self.build_body(data, &mut body);
            self.build_new_body(body);
        });
        if delete_index.is_some() {
            data.rest
                .request
                .body
                .body_form_data
                .remove(delete_index.unwrap());
        }
        if self.new_form.key != "" || self.new_form.value != "" || self.new_form.desc != "" {
            self.new_form.enable = true;
            data.rest
                .request
                .body
                .body_form_data
                .push(self.new_form.clone());
            self.new_form.key = "".to_string();
            self.new_form.value = "".to_string();
            self.new_form.desc = "".to_string();
            self.new_form.enable = false;
        }
    }
}

impl RequestBodyFormDataPanel {
    fn build_header(&self) -> fn(TableRow) {
        |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| {
                ui.strong("TYPE");
            });
            header.col(|ui| {
                ui.strong("KEY");
            });
            header.col(|ui| {
                ui.strong("VALUE");
            });
            header.col(|ui| {
                ui.strong("DESCRIPTION");
            });
        }
    }

    fn build_body(&self, data: &mut CentralRequestItem, mut body: &mut TableBody) -> Option<usize> {
        let mut delete_index: Option<usize> = None;
        for (index, param) in data.rest.request.body.body_form_data.iter_mut().enumerate() {
            body.row(18.0, |mut row| {
                row.col(|ui| {
                    ui.checkbox(&mut param.enable, "");
                });
                row.col(|ui| {
                    if ui.button("x").clicked() {
                        delete_index = Some(index)
                    }
                });
                row.col(|ui| {
                    egui::ComboBox::from_id_source(
                        "form_data_type_".to_string() + index.to_string().as_str(),
                    )
                    .selected_text(param.data_type.to_string())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for x in MultipartDataType::iter() {
                            ui.selectable_value(&mut param.data_type, x.clone(), x.to_string());
                        }
                    });
                });
                row.col(|ui| {
                    ui.text_edit_singleline(&mut param.key);
                });
                row.col(|ui| {
                    if param.data_type == MultipartDataType::Text {
                        ui.text_edit_singleline(&mut param.value);
                    } else {
                        let mut button_name =
                            utils::build_with_count_ui_header("Select File".to_string(), 0, ui);
                        if param.value != "" {
                            button_name =
                                utils::build_with_count_ui_header("Select File".to_string(), 1, ui);
                        }
                        if ui.button(button_name).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                param.value = path.display().to_string();
                            }
                        }
                    }
                });
                row.col(|ui| {
                    TextEdit::singleline(&mut param.desc)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            });
        }
        delete_index
    }

    fn build_new_body(&mut self, mut body: TableBody) {
        body.row(18.0, |mut row| {
            row.col(|ui| {
                ui.add_enabled(false, Checkbox::new(&mut self.new_form.enable, ""));
            });
            row.col(|ui| {
                ui.add_enabled(false, Button::new("x"));
            });
            row.col(|ui| {
                egui::ComboBox::from_id_source("form_data_type")
                    .selected_text(self.new_form.data_type.to_string())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for x in MultipartDataType::iter() {
                            ui.selectable_value(
                                &mut self.new_form.data_type,
                                x.clone(),
                                x.to_string(),
                            );
                        }
                    });
            });
            row.col(|ui| {
                ui.text_edit_singleline(&mut self.new_form.key);
            });
            row.col(|ui| {
                if self.new_form.data_type == MultipartDataType::Text {
                    ui.text_edit_singleline(&mut self.new_form.value);
                } else {
                    if ui.button("Select File").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.new_form.value = path.display().to_string()
                        }
                    }
                }
            });
            row.col(|ui| {
                TextEdit::singleline(&mut self.new_form.desc)
                    .desired_width(f32::INFINITY)
                    .ui(ui);
            });
        });
    }
}
