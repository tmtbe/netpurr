use std::cell::RefCell;
use std::rc::Rc;

use egui::Ui;

use crate::events::MailPost;
use crate::models::central_model::CentralRequestModels;
use crate::models::Model;
use crate::panels::View;

#[derive(PartialEq, Eq)]
enum Panel {
    Request,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Request
    }
}

#[derive(Default)]
pub struct MyCentralPanel {
    open_panel: Panel,
    model: CentralRequestModels,
}

impl View for MyCentralPanel {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        self.model.init(mail_post)
    }

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
        ui.horizontal(|ui| {
            for request_data in self.model.get_data().data_list {
                ui.selectable_value(&mut self.open_panel, Panel::Request, request_data.rest.request.method + &*request_data.rest.request.url);
            }
            if ui.button("+").clicked() {
                self.model.add_new()
            }
            if ui.button("...").clicked() {}
        });
    }
}
