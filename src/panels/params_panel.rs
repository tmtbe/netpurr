use eframe::emath::Align;
use egui::{Direction, Layout, Ui};
use egui_extras::{Column, TableBuilder};

use crate::data::{AppData, QueryParam};
use crate::panels::DataView;

#[derive(Default)]
pub struct ParamsPanel {
    new_param: QueryParam,
}

impl DataView for ParamsPanel {
    type CursorType = String;
    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let data = app_data
            .central_request_data_list
            .data_map
            .get_mut(cursor.as_str())
            .unwrap();
        ui.label("Query Params");
        let mut delete_index = None;
        let table = TableBuilder::new(ui)
            .resizable(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::exact(20.0))
            .column(Column::initial(200.0).range(40.0..=300.0))
            .column(Column::initial(200.0).range(40.0..=300.0))
            .column(Column::remainder())
            .min_scrolled_height(0.0);
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("");
                });
                header.col(|ui| {
                    ui.strong("");
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
            })
            .body(|mut body| {
                for (index, param) in data.rest.request.params.iter_mut().enumerate() {
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
                            ui.text_edit_singleline(&mut param.key);
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut param.value);
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut param.desc);
                        });
                    });
                }
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.checkbox(&mut self.new_param.enable, "");
                    });
                    row.col(|ui| ui.button("x").enabled = false);
                    row.col(|ui| {
                        ui.text_edit_singleline(&mut self.new_param.key);
                    });
                    row.col(|ui| {
                        ui.text_edit_singleline(&mut self.new_param.value);
                    });
                    row.col(|ui| {
                        ui.text_edit_singleline(&mut self.new_param.desc);
                    });
                });
            });
        if delete_index.is_some() {
            data.rest.request.params.remove(delete_index.unwrap());
        }
        if self.new_param.key != "" || self.new_param.value != "" || self.new_param.desc != "" {
            data.rest.request.params.push(self.new_param.clone());
            self.new_param.key = "".to_string();
            self.new_param.value = "".to_string();
            self.new_param.desc = "".to_string();
            self.new_param.enable = false;
        }
    }
}
