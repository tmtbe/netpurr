use egui::CollapsingHeader;

use crate::data::central_request_data::CentralRequestItem;
use crate::data::workspace::WorkspaceData;
use crate::operation::Operation;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct HistoryPanel {}

impl DataView for HistoryPanel {
    type CursorType = i32;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        for (date, date_history_data) in workspace_data.history_data_list.get_group().iter().rev() {
            CollapsingHeader::new(date.to_string())
                .default_open(false)
                .show(ui, |ui| {
                    for history_rest_item in date_history_data.history_list.iter().rev() {
                        let lb =
                            utils::build_rest_ui_header(history_rest_item.rest.clone(), None, ui);
                        let button = ui.button(lb);
                        if button.clicked() {
                            workspace_data.add_crt(CentralRequestItem {
                                id: history_rest_item.id.clone(),
                                collection_path: None,
                                rest: history_rest_item.rest.clone(),
                                ..Default::default()
                            })
                        }
                    }
                });
        }
    }
}
