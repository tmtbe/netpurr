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

impl DataView for MyLeftPanel {
    type CursorType = i32;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(egui::TextEdit::singleline(&mut self.filter).desired_width(120.0));
            if ui.button("ｘ").clicked() {
                self.filter.clear();
            }
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.open_panel, Panel::History, "History");
            ui.selectable_value(&mut self.open_panel, Panel::Collections, "Collections");
        });
        ScrollArea::vertical().show(ui, |ui| match self.open_panel {
            Panel::History => {
                self.history_panel
                    .set_and_render(ui, operation, workspace_data, 0);
            }
            Panel::Collections => {
                self.collections_panel
                    .set_and_render(ui, operation, workspace_data, 0);
            }
        });
    }
}
