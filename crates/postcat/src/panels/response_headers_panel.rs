use eframe::emath::Align;
use egui::{Checkbox, Layout, TextEdit, Widget};
use egui_extras::{Column, TableBuilder};

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::DataView;

#[derive(Default)]
pub struct ResponseHeadersPanel {}

impl DataView for ResponseHeadersPanel {
    type CursorType = String;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: Self::CursorType,
    ) {
        let crt = workspace_data.must_get_crt(crt_id.clone());
        ui.label("Headers");
        ui.push_id("response_headers_table", |ui| {
            let table = TableBuilder::new(ui)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto())
                .column(Column::exact(20.0))
                .column(Column::initial(200.0).range(40.0..=300.0))
                .column(Column::remainder())
                .min_scrolled_height(200.0);
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
                })
                .body(|mut body| {
                    workspace_data.get_mut_crt(crt_id.clone(), |crt| {
                        for (_, header) in crt.rest.response.headers.iter_mut().enumerate() {
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.add_enabled(false, Checkbox::new(&mut header.enable, ""));
                                });
                                row.col(|ui| {});
                                row.col(|ui| {
                                    ui.text_edit_singleline(&mut header.key.clone());
                                });
                                row.col(|ui| {
                                    TextEdit::singleline(&mut header.value.clone())
                                        .desired_width(f32::INFINITY)
                                        .ui(ui);
                                });
                            });
                        }
                    });
                });
        });
    }
}
