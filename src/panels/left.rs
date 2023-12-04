use std::cell::RefCell;
use std::rc::Rc;

use egui::Ui;

use crate::events::MailPost;
use crate::panels::collections::CollectionsPanel;
use crate::panels::history::HistoryPanel;
use crate::panels::View;

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

impl View for MyLeftPanel {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        self.history_panel.init(mail_post.clone());
        self.collections_panel.init(mail_post.clone());
    }

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
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
                self.history_panel.render(ui, mail_post);
            }
            Panel::Collections => {
                self.collections_panel.render(ui, mail_post);
            }
        }
    }
}
