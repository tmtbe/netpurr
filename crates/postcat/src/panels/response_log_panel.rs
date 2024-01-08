use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::DataView;

#[derive(Default)]
pub struct ResponseLogPanel {}

impl DataView for ResponseLogPanel {
    type CursorType = String;

    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: Self::CursorType,
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
                for log in crt.rest.response.logger.logs.iter() {
                    let mut content = format!("> {}", log.show());
                    egui::TextEdit::multiline(&mut content)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(1)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter)
                        .show(ui);
                }
            });
        });
    }
}
