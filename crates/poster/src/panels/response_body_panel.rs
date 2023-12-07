use egui::{TextBuffer, Ui};

use crate::data::{AppData, Response};
use crate::panels::DataView;

#[derive(Default)]
pub struct ResponseBodyPanel {}

impl ResponseBodyPanel {
    fn get_language(response: &Response) -> String {
        let content_type_header = response.headers.iter().find(|h| h.key == "content-type");
        if content_type_header.is_some() {
            let content_type = content_type_header.unwrap().value.clone();
            if content_type.contains("json") {
                return "toml".to_string();
            } else if content_type.contains("html") {
                return "toml".to_string();
            } else if content_type.contains("js") {
                return "toml".to_string();
            } else if content_type.contains("xml") {
                return "toml".to_string();
            }
        }
        "toml".to_string()
    }
}

impl DataView for ResponseBodyPanel {
    type CursorType = String;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let data = app_data
            .central_request_data_list
            .data_map
            .get(cursor.as_str())
            .unwrap();
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                &theme,
                string,
                ResponseBodyPanel::get_language(&data.rest.response).as_str(),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        match String::from_utf8(data.rest.response.body.clone()) {
            Ok(s) => {
                ui.push_id("response_body", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut s.as_str())
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter),
                        );
                    });
                });
            }
            Err(e) => {
                ui.centered_and_justified(|ui| {
                    ui.label("Error String");
                });
            }
        }
    }
}
