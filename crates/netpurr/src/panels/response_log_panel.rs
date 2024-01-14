use egui_code_editor::{CodeEditor, ColorTheme};

use crate::data::workspace_data::WorkspaceData;
use crate::widgets::syntax::log_syntax;

#[derive(Default)]
pub struct ResponseLogPanel {}

impl ResponseLogPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let crt = workspace_data.must_get_crt(crt_id.clone());
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "log");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        ui.push_id("log_info", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, log) in crt.rest.response.logger.logs.iter().enumerate() {
                    let mut content = format!("> {}", log.show());
                    let mut code_editor = CodeEditor::default()
                        .id_source(format!("{}-{}", "log", index))
                        .with_rows(1)
                        .with_ui_fontsize(ui)
                        .with_syntax(log_syntax())
                        .with_numlines(false);
                    if ui.visuals().dark_mode {
                        code_editor = code_editor.with_theme(ColorTheme::GRUVBOX)
                    } else {
                        code_editor = code_editor.with_theme(ColorTheme::GRUVBOX_LIGHT)
                    }
                    code_editor
                        .show(ui, &mut content)
                        .response
                        .on_hover_text(content);
                }
            });
        });
    }
}
