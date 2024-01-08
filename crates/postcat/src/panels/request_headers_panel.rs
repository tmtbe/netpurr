use std::collections::BTreeMap;

use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextEdit, Widget};
use egui_extras::{Column, TableBody, TableBuilder};

use crate::data::central_request_data::CentralRequestItem;
use crate::data::environment::EnvironmentItemValue;
use crate::data::http::{Header, LockWith};
use crate::data::workspace::WorkspaceData;
use crate::operation::Operation;
use crate::panels::DataView;
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct RequestHeadersPanel {
    new_header: Header,
}

impl DataView for RequestHeadersPanel {
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
            ui.label("Headers");
            let mut delete_index = None;
            ui.push_id("request_headers_table", |ui| {
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
            });
            if delete_index.is_some() {
                crt.rest.request.headers.remove(delete_index.unwrap());
            }
            if self.new_header.key != ""
                || self.new_header.value != ""
                || self.new_header.desc != ""
            {
                self.new_header.enable = true;
                crt.rest.request.headers.push(self.new_header.clone());
                self.new_header.key = "".to_string();
                self.new_header.value = "".to_string();
                self.new_header.desc = "".to_string();
                self.new_header.enable = false;
            }
        });
    }
}

impl RequestHeadersPanel {
    fn build_body(
        &self,
        data: &mut CentralRequestItem,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        mut body: &mut TableBody,
    ) -> Option<usize> {
        let mut delete_index = None;
        for (index, header) in data.rest.request.headers.iter_mut().enumerate() {
            body.row(18.0, |mut row| {
                row.col(|ui| {
                    ui.add_enabled(
                        header.lock_with == LockWith::NoLock,
                        Checkbox::new(&mut header.enable, ""),
                    );
                });
                row.col(|ui| {
                    if ui
                        .add_enabled(header.lock_with == LockWith::NoLock, Button::new("x"))
                        .clicked()
                    {
                        delete_index = Some(index)
                    }
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .enable(header.lock_with == LockWith::NoLock)
                        .all_space(false)
                        .build(
                            "request_header_key_".to_string() + index.to_string().as_str(),
                            &mut header.key,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .enable(header.lock_with == LockWith::NoLock)
                        .all_space(false)
                        .build(
                            "request_header_value_".to_string() + index.to_string().as_str(),
                            &mut header.value,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    ui.add_enabled(
                        header.lock_with == LockWith::NoLock,
                        TextEdit::singleline(&mut header.desc).desired_width(f32::INFINITY),
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
                ui.add_enabled(false, Checkbox::new(&mut self.new_header.enable, ""));
            });
            row.col(|ui| {
                ui.add_enabled(false, Button::new("x"));
            });
            row.col(|ui| {
                HighlightTemplateSinglelineBuilder::default()
                    .envs(envs.clone())
                    .enable(self.new_header.lock_with == LockWith::NoLock)
                    .all_space(false)
                    .build(
                        "request_header_key_new".to_string(),
                        &mut self.new_header.key,
                    )
                    .ui(ui);
            });
            row.col(|ui| {
                HighlightTemplateSinglelineBuilder::default()
                    .envs(envs.clone())
                    .enable(self.new_header.lock_with == LockWith::NoLock)
                    .all_space(false)
                    .build(
                        "request_header_value_new".to_string(),
                        &mut self.new_header.value,
                    )
                    .ui(ui);
            });
            row.col(|ui| {
                ui.add_enabled_ui(self.new_header.lock_with == LockWith::NoLock, |ui| {
                    TextEdit::singleline(&mut self.new_header.desc)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            });
        });
    }
}
