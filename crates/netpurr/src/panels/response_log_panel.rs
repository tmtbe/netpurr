use egui_code_editor::highlighting::highlight;
use egui_code_editor::{CodeEditor, ColorTheme};

use netpurr_core::data::http::Response;

use crate::widgets::syntax::js_syntax;

#[derive(Default)]
pub struct ResponseLogPanel {}

impl ResponseLogPanel {
    pub fn set_and_render(&mut self, ui: &mut egui::Ui, response: &Response) {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "js");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        ui.push_id("log_info", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, log) in response.logger.logs.iter().enumerate() {
                    let mut content = format!("> {}", log.show());
                    egui::TextEdit::multiline(&mut content)
                        .layouter(&mut layouter)
                        .lock_focus(true)
                        .desired_rows(4)
                        .frame(true)
                        .desired_width(f32::MAX)
                        .show(ui);
                }
            });
        });
    }
}
