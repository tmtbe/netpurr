use egui::{TextEdit, Ui, Widget};

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;

#[derive(Default)]
pub struct OpenApiEditorPanel {
    source: String,
    collection_name: String,
}

impl OpenApiEditorPanel {
    pub fn render(
        &mut self,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
        ui: &mut Ui,
    ) {
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 30.0)
            .show(ui, |ui| {
                let collection_name = config_data.select_collection().unwrap_or_default();
                let collection = workspace_data.get_collection_by_name(collection_name.clone());
                if self.collection_name != collection_name {
                    self.collection_name = collection_name.clone();
                    self.source = collection
                        .map(|c| serde_yaml::to_string(&c.openapi).unwrap_or_default())
                        .unwrap_or_default();
                }
                let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
                let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
                    let mut layout_job = egui_extras::syntax_highlighting::highlight(
                        ui.ctx(),
                        &theme,
                        string,
                        "yaml",
                    );
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };
                TextEdit::multiline(&mut self.source)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter)
                    .ui(ui)
            });
    }
}
