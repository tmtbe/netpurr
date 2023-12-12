use egui::Ui;
use strum::IntoEnumIterator;

use crate::data::{Auth, AuthType};
use crate::panels::AlongDataView;
use crate::utils;

#[derive(Default)]
pub struct AuthPanel {}

impl AlongDataView for AuthPanel {
    type DataType = Auth;

    fn set_and_render(&mut self, data: &mut Self::DataType, ui: &mut Ui) {
        utils::left_right_panel(
            ui,
            "auth_left",
            |ui| {
                ui.strong("AUTH");
                ui.label("The authorization header will be automatically generated when you send the request. ");
                egui::ComboBox::from_id_source("method")
                    .selected_text(data.auth_type.to_string())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for x in AuthType::iter() {
                            ui.selectable_value(&mut data.auth_type, x.clone(), x.to_string());
                        }
                    });
            },
            "auth_right",
            |ui| {},
        );
    }
}
