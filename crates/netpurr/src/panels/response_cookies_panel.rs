use std::collections::BTreeMap;

use eframe::emath::Align;
use egui::{Layout, TextEdit, Widget};
use egui_extras::{Column, TableBuilder};

use crate::data::cookies_manager::Cookie;

#[derive(Default)]
pub struct ResponseCookiesPanel {}

impl ResponseCookiesPanel {
    pub fn set_and_render(&mut self, ui: &mut egui::Ui, cookies: &BTreeMap<String, Cookie>) {
        ui.label("Cookies");
        ui.push_id("response_cookies_table", |ui| {
            let table = TableBuilder::new(ui)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::initial(200.0).range(100.0..=300.0))
                .column(Column::initial(200.0).range(100.0..=300.0))
                .column(Column::initial(200.0).range(100.0..=300.0))
                .column(Column::initial(50.0).range(50.0..=100.0))
                .column(Column::initial(100.0).range(50.0..=100.0))
                .column(Column::initial(50.0).range(50.0..=100.0))
                .column(Column::remainder())
                .min_scrolled_height(200.0);
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Value");
                    });
                    header.col(|ui| {
                        ui.strong("Domain");
                    });
                    header.col(|ui| {
                        ui.strong("Path");
                    });
                    header.col(|ui| {
                        ui.strong("Expires");
                    });
                    header.col(|ui| {
                        ui.strong("HttpOnly");
                    });
                    header.col(|ui| {
                        ui.strong("Secure");
                    });
                })
                .body(|mut body| {
                    for (_, cookie) in cookies {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut cookie.name.clone());
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut cookie.value.clone());
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut cookie.domain.clone());
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut cookie.path.clone());
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut cookie.expires.clone());
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut cookie.http_only.to_string());
                            });
                            row.col(|ui| {
                                TextEdit::singleline(&mut cookie.secure.to_string())
                                    .desired_width(f32::INFINITY)
                                    .ui(ui);
                            });
                        });
                    }
                });
        });
    }
}
