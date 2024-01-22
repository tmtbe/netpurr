use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::response_panel::ResponsePanel;

#[derive(Default)]
pub struct RightPanel {
    response_panel: ResponsePanel,
}

impl RightPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        match &workspace_data.get_crt_select_id() {
            None => {}
            Some(crt_id) => {
                self.response_panel
                    .set_and_render(ui, operation, workspace_data, crt_id.clone());
            }
        }
    }
}
