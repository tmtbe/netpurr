use crate::data::workspace_data::WorkspaceData;

#[derive(Default)]
pub struct ResponseHeadersPanel {}

impl ResponseHeadersPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
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
                for header in crt.record.must_get_mut_rest().response.headers.iter() {
                    ui.label(header.key.clone());
                    ui.label(header.value.clone());
                    ui.end_row();
                }
            });
    }
}
