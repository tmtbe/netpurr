use egui::{Image, TextBuffer};
use uuid::Uuid;

use netpurr_core::data::http::{Header, Response};

use crate::operation::operation::Operation;
use crate::windows::view_json_windows::ViewJsonWindows;

#[derive(Default)]
pub struct ResponseBodyPanel {}

impl ResponseBodyPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &Operation,
        response: &Response,
    ) {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                &theme,
                string,
                ResponseBodyPanel::get_language(response).as_str(),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        match self.get_response_content_type(response) {
            None => {}
            Some(content_type) => {
                if content_type.value.starts_with("image") {
                    let image = Image::from_bytes(
                        response.request.get_url_with_schema(),
                        response.body.to_vec(),
                    );
                    ui.add(image);
                } else {
                    match String::from_utf8(response.body.to_vec()) {
                        Ok(s) => {
                            ui.horizontal(|ui| {
                                let tooltip = "Click to copy the response body";
                                if ui.button("📋").on_hover_text(tooltip).clicked() {
                                    ui.output_mut(|o| o.copied_text = s.to_owned());
                                    operation.add_success_toast("Copy success");
                                }
                                if content_type.value.contains("json") {
                                    if ui.button("View Json Tree").clicked() {
                                        operation.add_window(Box::new(
                                            ViewJsonWindows::default()
                                                .with_json(s.clone(), Uuid::new_v4().to_string()),
                                        ))
                                    }
                                }
                            });
                            let mut content = s;
                            ui.push_id("response_body", |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut content)
                                            .font(egui::TextStyle::Monospace) // for cursor height
                                            .code_editor()
                                            .desired_rows(12)
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
        }
    }
    pub fn get_response_content_type(&self, response: &Response) -> Option<Header> {
        response
            .headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type")
            .cloned()
    }
    fn get_language(response: &Response) -> String {
        match response.headers.iter().find(|h| h.key == "content-type") {
            None => "json".to_string(),
            Some(content_type_header) => {
                let content_type = content_type_header.value.clone();
                if content_type.contains("json") {
                    return "json".to_string();
                } else if content_type.contains("html") {
                    return "html".to_string();
                } else if content_type.contains("js") {
                    return "js".to_string();
                } else if content_type.contains("xml") {
                    return "xml".to_string();
                } else {
                    "json".to_string()
                }
            }
        }
    }
}
