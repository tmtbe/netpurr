use egui::TextBuffer;

use crate::data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::DataView;

#[derive(Default)]
pub struct TestResultPanel {}

impl DataView for TestResultPanel {
    type CursorType = String;

    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        let data = workspace_data
            .central_request_data_list
            .data_map
            .get(cursor.as_str())
            .unwrap();
        ui.push_id("test_info", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for test_info in data.test_result.test_info_list.iter() {
                    let mut content = test_info.name.clone();
                    egui::TextEdit::multiline(&mut content)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(1)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .show(ui);
                }
            });
        });
    }
}
