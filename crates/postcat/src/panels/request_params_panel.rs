use std::collections::BTreeMap;

use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextEdit, Widget};
use egui_extras::{Column, TableBody, TableBuilder};

use crate::data::central_request_data::CentralRequestItem;
use crate::data::environment::EnvironmentItemValue;
use crate::data::http::{LockWith, QueryParam};
use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::DataView;
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct RequestParamsPanel {
    new_param: QueryParam,
}

impl DataView for RequestParamsPanel {
    type CursorType = String;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: Self::CursorType,
    ) {
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        workspace_data.get_mut_crt(crt_id.clone(), |crt| {
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
                    delete_index = self.build_body(crt, &envs, &mut body);
                    self.build_new_body(envs, body);
                });
            if delete_index.is_some() {
                crt.rest.request.params.remove(delete_index.unwrap());
            }
            if self.new_param.key != "" || self.new_param.value != "" || self.new_param.desc != "" {
                self.new_param.enable = true;
                crt.rest.request.params.push(self.new_param.clone());
                self.new_param.key = "".to_string();
                self.new_param.value = "".to_string();
                self.new_param.desc = "".to_string();
                self.new_param.enable = false;
            }
        });
    }
}

impl RequestParamsPanel {
    fn build_body(
        &self,
        data: &mut CentralRequestItem,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        body: &mut TableBody,
    ) -> Option<usize> {
        let mut delete_index = None;
        for (index, param) in data.rest.request.params.iter_mut().enumerate() {
            body.row(18.0, |mut row| {
                row.col(|ui| {
                    ui.add_enabled(
                        param.lock_with == LockWith::NoLock,
                        Checkbox::new(&mut param.enable, ""),
                    );
                });
                row.col(|ui| {
                    if ui
                        .add_enabled(param.lock_with == LockWith::NoLock, Button::new("x"))
                        .clicked()
                    {
                        delete_index = Some(index)
                    }
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .enable(param.lock_with == LockWith::NoLock)
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
                        .enable(param.lock_with == LockWith::NoLock)
                        .all_space(false)
                        .build(
                            "request_parmas_value_".to_string() + index.to_string().as_str(),
                            &mut param.value,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    ui.add_enabled(
                        param.lock_with == LockWith::NoLock,
                        TextEdit::singleline(&mut param.desc).desired_width(f32::INFINITY),
                    );
                });
            });
        }
        delete_index
    }

    fn build_new_body(
        &mut self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        mut body: TableBody,
    ) {
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
                    .enable(self.new_param.lock_with == LockWith::NoLock)
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
                    .enable(self.new_param.lock_with == LockWith::NoLock)
                    .build(
                        "request_parmas_value_new".to_string(),
                        &mut self.new_param.value,
                    )
                    .ui(ui);
            });
            row.col(|ui| {
                ui.add_enabled_ui(self.new_param.lock_with == LockWith::NoLock, |ui| {
                    TextEdit::singleline(&mut self.new_param.desc)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            });
        });
    }
}
