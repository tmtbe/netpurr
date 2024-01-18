use egui::ScrollArea;

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::panels::collections_panel::CollectionsPanel;
use crate::panels::history_panel::HistoryPanel;
use crate::panels::DataView;

#[derive(PartialEq, Eq)]
enum Panel {
    History,
    Collections,
}

impl Default for Panel {
    fn default() -> Self {
        Self::History
    }
}

#[derive(Default)]
pub struct MyLeftPanel {
    history_panel: HistoryPanel,
    collections_panel: CollectionsPanel,
    open_panel: Panel,
    filter: String,
}

impl MyLeftPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.open_panel, Panel::History, "History");
            ui.selectable_value(&mut self.open_panel, Panel::Collections, "Collections");
        });
        ScrollArea::vertical().show(ui, |ui| match self.open_panel {
            Panel::History => {
                self.history_panel.set_and_render(ui, workspace_data);
            }
            Panel::Collections => {
                self.collections_panel
                    .set_and_render(ui, operation, workspace_data);
            }
        });
    }
}
