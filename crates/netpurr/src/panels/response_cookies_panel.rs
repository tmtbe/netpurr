use std::collections::BTreeMap;

use netpurr_core::data::cookies_manager::Cookie;

#[derive(Default)]
pub struct ResponseCookiesPanel {}

impl ResponseCookiesPanel {
    pub fn set_and_render(&mut self, ui: &mut egui::Ui, cookies: &BTreeMap<String, Cookie>) {
        ui.label("Cookies");
        egui::Grid::new("response_cookies_grid")
            .striped(true)
            .min_col_width(10.0)
            .max_col_width(100.0)
            .show(ui, |ui| {
                ui.strong("Name");
                ui.strong("Value");
                ui.strong("Domain");
                ui.strong("Path");
                ui.strong("Expires");
                ui.strong("HttpOnly");
                ui.strong("Secure");
                ui.end_row();
                for (_, cookie) in cookies {
                    ui.label(cookie.name.clone());
                    ui.label(cookie.value.clone());
                    ui.label(cookie.domain.clone());
                    ui.label(cookie.path.clone());
                    ui.label(cookie.expires.clone());
                    ui.label(cookie.http_only.to_string());
                    ui.label(cookie.secure.to_string());
                    ui.end_row();
                }
            });
    }
}
