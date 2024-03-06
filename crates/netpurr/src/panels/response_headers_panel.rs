use netpurr_core::data::http::Response;

#[derive(Default)]
pub struct ResponseHeadersPanel {}

impl ResponseHeadersPanel {
    pub fn set_and_render(&mut self, ui: &mut egui::Ui, response: &Response) {
        ui.label("Headers");
        egui::Grid::new("response_headers_grid")
            .striped(true)
            .min_col_width(100.0)
            .max_col_width(ui.available_width())
            .num_columns(2)
            .show(ui, |ui| {
                ui.strong("Key");
                ui.strong("Value");
                ui.end_row();
                for header in response.headers.iter() {
                    ui.label(header.key.clone());
                    ui.label(header.value.clone());
                    ui.end_row();
                }
            });
    }
}
