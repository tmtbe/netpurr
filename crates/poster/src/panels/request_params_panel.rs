use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBuilder};

use crate::data::{AppData, QueryParam};
use crate::panels::DataView;
use crate::widgets::highlight_template_singleline::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct RequestParamsPanel {
    new_param: QueryParam,
}

impl DataView for RequestParamsPanel {
    type CursorType = String;
    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let (data, envs) = app_data.get_mut_crt_and_envs(cursor.clone());
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
            .max_scroll_height(100.0);
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
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(envs.clone())
                                .all_space(false)
                                .build(
                                    "request_parmas_key_".to_string() + index.to_string().as_str(),
                                    &mut param.key,
                                )
                                .ui(ui);
                        });
                        row.col(|ui| {
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(envs.clone())
                                .all_space(false)
                                .build(
                                    "request_parmas_value_".to_string()
                                        + index.to_string().as_str(),
                                    &mut param.value,
                                )
                                .ui(ui);
                        });
                        row.col(|ui| {
                            TextEdit::singleline(&mut param.desc)
                                .desired_width(f32::INFINITY)
                                .ui(ui);
                        });
                    });
                }
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.add_enabled(false, Checkbox::new(&mut self.new_param.enable, ""));
                    });
                    row.col(|ui| {
                        ui.add_enabled(false, Button::new("x"));
                    });
                    row.col(|ui| {
                        HighlightTemplateSinglelineBuilder::default()
                            .envs(envs.clone())
                            .all_space(false)
                            .build(
                                "request_parmas_key_new".to_string(),
                                &mut self.new_param.key,
                            )
                            .ui(ui);
                    });
                    row.col(|ui| {
                        HighlightTemplateSinglelineBuilder::default()
                            .envs(envs.clone())
                            .all_space(false)
                            .build(
                                "request_parmas_value_new".to_string(),
                                &mut self.new_param.value,
                            )
                            .ui(ui);
                    });
                    row.col(|ui| {
                        TextEdit::singleline(&mut self.new_param.desc)
                            .desired_width(f32::INFINITY)
                            .ui(ui);
                    });
                });
            });
        if delete_index.is_some() {
            data.rest.request.params.remove(delete_index.unwrap());
        }
        if self.new_param.key != "" || self.new_param.value != "" || self.new_param.desc != "" {
            self.new_param.enable = true;
            data.rest.request.params.push(self.new_param.clone());
            self.new_param.key = "".to_string();
            self.new_param.value = "".to_string();
            self.new_param.desc = "".to_string();
            self.new_param.enable = false;
        }
    }
}
