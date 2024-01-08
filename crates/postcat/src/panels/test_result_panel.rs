use crate::data::workspace::WorkspaceData;
use crate::operation::Operation;
use crate::panels::{DataView, HORIZONTAL_GAP};

#[derive(Default)]
pub struct TestResultPanel {}

impl DataView for TestResultPanel {
    type CursorType = String;

    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: Self::CursorType,
    ) {
        let crt = workspace_data.must_get_crt(crt_id.clone());
        ui.push_id("test_info", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for test_info in crt.test_result.test_info_list.iter() {
                    ui.horizontal(|ui| {
                        ui.add_space(HORIZONTAL_GAP * 2.0);
                        ui.strong(test_info.status.to_string());
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.label(test_info.name.clone());
                            for tar in test_info.results.iter() {
                                ui.horizontal(|ui| {
                                    ui.add_space(HORIZONTAL_GAP * 2.0);
                                    ui.separator();
                                    ui.strong(tar.assert_result.to_string());
                                    ui.label(tar.msg.to_string());
                                });
                            }
                        });
                    });
                }
            });
        });
    }
}
