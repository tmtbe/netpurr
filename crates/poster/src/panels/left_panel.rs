use egui::Ui;

use crate::data::AppData;
use crate::panels::collections_panel::CollectionsPanel;
use crate::panels::DataView;
use crate::panels::history_panel::HistoryPanel;

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
    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
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

        match self.open_panel {
            Panel::History => {
                self.history_panel.set_and_render(app_data, 0, ui);
            }
            Panel::Collections => {
                self.collections_panel.set_and_render(app_data, 0, ui);
            }
        }
    }
}
