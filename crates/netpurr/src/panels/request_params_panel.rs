use std::collections::BTreeMap;

use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBody, TableBuilder};

use netpurr_core::data::environment::EnvironmentItemValue;
use netpurr_core::data::http::{LockWith, QueryParam};

use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;
use netpurr_core::data::central_request_data::CentralRequestItem;
use netpurr_core::data::workspace_data::WorkspaceData;

#[derive(Default)]
pub struct RequestParamsPanel {
    new_query_param: QueryParam,
}

impl RequestParamsPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
            self.render_query_params(ui, &envs, crt);
            let get_path_variable_keys =
                crt.record.must_get_rest().request.get_path_variable_keys();
            if !get_path_variable_keys.is_empty() {
                self.render_path_variables(ui, &envs, crt);
            }
        });
    }

    fn render_query_params(
        &mut self,
        ui: &mut Ui,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        crt: &mut CentralRequestItem,
    ) {
        ui.label("Query Params");
        let mut delete_index = None;
        ui.push_id("query_params_table", |ui| {
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
                    delete_index = self.build_query_params_body(crt, &envs, &mut body);
                    self.build_new_query_params_body(envs, body);
                });
        });
        if delete_index.is_some() {
            crt.record
                .must_get_mut_rest()
                .request
                .params
                .remove(delete_index.unwrap());
        }
        if self.new_query_param.key != ""
            || self.new_query_param.value != ""
            || self.new_query_param.desc != ""
        {
            self.new_query_param.enable = true;
            crt.record
                .must_get_mut_rest()
                .request
                .params
                .push(self.new_query_param.clone());
            self.new_query_param.key = "".to_string();
            self.new_query_param.value = "".to_string();
            self.new_query_param.desc = "".to_string();
            self.new_query_param.enable = false;
        }
    }

    fn render_path_variables(
        &mut self,
        ui: &mut Ui,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        crt: &mut CentralRequestItem,
    ) {
        ui.label("Path Variables");
        ui.push_id("path_variables_table", |ui| {
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
                    self.build_path_variables_body(crt, &envs, &mut body);
                });
        });
    }
}

impl RequestParamsPanel {
    fn build_query_params_body(
        &self,
        data: &mut CentralRequestItem,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        body: &mut TableBody,
    ) -> Option<usize> {
        let mut delete_index = None;
        for (index, param) in data
            .record
            .must_get_mut_rest()
            .request
            .params
            .iter_mut()
            .enumerate()
        {
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
    fn build_path_variables_body(
        &self,
        data: &mut CentralRequestItem,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        body: &mut TableBody,
    ) {
        for (index, path_variable) in data
            .record
            .must_get_mut_rest()
            .request
            .path_variables
            .iter_mut()
            .enumerate()
        {
            body.row(18.0, |mut row| {
                row.col(|ui| {
                    let mut enable = true;
                    ui.add_enabled(false, Checkbox::new(&mut enable, ""));
                });
                row.col(|ui| {
                    ui.add_enabled(false, Button::new("x"));
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .enable(false)
                        .all_space(false)
                        .build(
                            "path_variable_key_".to_string() + index.to_string().as_str(),
                            &mut path_variable.key,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .enable(true)
                        .all_space(false)
                        .build(
                            "path_variable_value_".to_string() + index.to_string().as_str(),
                            &mut path_variable.value,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    TextEdit::singleline(&mut path_variable.desc)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            });
        }
    }

    fn build_new_query_params_body(
        &mut self,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        mut body: TableBody,
    ) {
        body.row(18.0, |mut row| {
            row.col(|ui| {
                ui.add_enabled(false, Checkbox::new(&mut self.new_query_param.enable, ""));
            });
            row.col(|ui| {
                ui.add_enabled(false, Button::new("x"));
            });
            row.col(|ui| {
                HighlightTemplateSinglelineBuilder::default()
                    .envs(envs.clone())
                    .all_space(false)
                    .enable(self.new_query_param.lock_with == LockWith::NoLock)
                    .build(
                        "request_params_key_new".to_string(),
                        &mut self.new_query_param.key,
                    )
                    .ui(ui);
            });
            row.col(|ui| {
                HighlightTemplateSinglelineBuilder::default()
                    .envs(envs.clone())
                    .all_space(false)
                    .enable(self.new_query_param.lock_with == LockWith::NoLock)
                    .build(
                        "request_params_value_new".to_string(),
                        &mut self.new_query_param.value,
                    )
                    .ui(ui);
            });
            row.col(|ui| {
                ui.add_enabled_ui(self.new_query_param.lock_with == LockWith::NoLock, |ui| {
                    TextEdit::singleline(&mut self.new_query_param.desc)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            });
        });
    }
}
