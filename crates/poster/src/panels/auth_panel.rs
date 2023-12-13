use std::collections::BTreeMap;

use egui::{Ui, Widget};
use strum::IntoEnumIterator;

use crate::data::{Auth, AuthType};
use crate::panels::highlight_template_singleline::HighlightTemplateSinglelineBuilder;
use crate::panels::{AlongDataView, HORIZONTAL_GAP, VERTICAL_GAP};

#[derive(Default)]
pub struct AuthPanel {
    envs: BTreeMap<String, String>,
}

impl AuthPanel {
    pub fn set_envs(&mut self, envs: BTreeMap<String, String>) {
        self.envs = envs;
    }
}

impl AlongDataView for AuthPanel {
    type DataType = Auth;

    fn set_and_render(&mut self, data: &mut Self::DataType, ui: &mut Ui) {
        ui.horizontal(|ui| {
            egui::SidePanel::left("auth_left")
                .resizable(true)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.strong("AUTH");
                    ui.add_space(VERTICAL_GAP);
                    ui.label("The authorization header will be automatically generated when you send the request. ");
                    ui.add_space(VERTICAL_GAP);
                    egui::ComboBox::from_id_source("method")
                        .selected_text(data.auth_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            for x in AuthType::iter() {
                                ui.selectable_value(&mut data.auth_type, x.clone(), x.to_string());
                            }
                        });
                    ui.add_space(VERTICAL_GAP);
                });
            egui::SidePanel::right("auth_right")
                .resizable(true)
                .show_separator_line(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| match data.auth_type {
                    AuthType::NoAuth => {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(VERTICAL_GAP * 5.0);
                            ui.label("This request does not use any authorization. ");
                            ui.add_space(VERTICAL_GAP * 5.0);
                        });
                    }
                    AuthType::BearerToken => {
                        ui.add_space(VERTICAL_GAP * 5.0);
                        ui.horizontal(|ui| {
                            ui.add_space(HORIZONTAL_GAP);
                            ui.label("Token:");
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(self.envs.clone())
                                .build("token".to_string(), &mut data.bearer_token)
                                .ui(ui);
                            ui.add_space(HORIZONTAL_GAP);
                        });
                        ui.add_space(VERTICAL_GAP * 5.0);
                    }
                    AuthType::BasicAuth => {
                        ui.add_space(VERTICAL_GAP * 2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(HORIZONTAL_GAP);
                            ui.label("Username:");
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(self.envs.clone())
                                .build("username".to_string(), &mut data.basic_username)
                                .ui(ui);
                        });
                        ui.add_space(VERTICAL_GAP);
                        ui.horizontal(|ui| {
                            ui.add_space(HORIZONTAL_GAP);
                            ui.label("Password: ");
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(self.envs.clone())
                                .build("password".to_string(), &mut data.basic_password)
                                .ui(ui);
                        });
                        ui.add_space(VERTICAL_GAP * 2.0);
                    }
                });
        });
    }
}
