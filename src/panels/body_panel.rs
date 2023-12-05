use crate::data::AppData;
use crate::panels::DataView;
use egui::{TextBuffer, Ui};

#[derive(Default)]
pub struct BodyPanel {}
impl DataView for BodyPanel {
    type CursorType = String;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let data = app_data
            .central_request_data_list
            .data_map
            .get(cursor.as_str())
            .unwrap();
        match String::from_utf8(data.rest.response.body.clone()) {
            Ok(s) => {
                ui.push_id("response_body", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui_extras::syntax_highlighting::code_view_ui(
                            ui,
                            &theme,
                            s.as_str(),
                            "rs",
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
