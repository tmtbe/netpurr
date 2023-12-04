use std::cell::RefCell;
use std::rc::Rc;

use egui::Ui;

use crate::events::MailPost;
use crate::models::central_model::{CentralRequestData, CentralRequestModels, Panel};
use crate::models::http::HttpRecord;
use crate::models::Model;
use crate::panels::editor_panel::EditorPanel;
use crate::panels::View;


#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel:EditorPanel,
    model: CentralRequestModels,
}

impl View for MyCentralPanel {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        self.model.init(mail_post)
    }

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
        ui.horizontal(|ui| {
            for request_data in self.model.get_data().data_list {
                ui.selectable_value(&mut self.model.open_panel, Panel::RequestId(request_data.id),
                                    request_data.rest.request.method + &*request_data.rest.request.url);
            }
            if ui.button("+").clicked() {
                self.model.add_new()
            }
            if ui.button("...").clicked() {}
        });
        match self.model.open_panel.clone() {
            Panel::RequestId(request_id) => {
                self.model.get_data().data_list.iter().find(|c| {
                    c.id == request_id
                }).map(|c|{
                    self.editor_panel.set(c.to_owned());
                    self.editor_panel.render(ui, mail_post);
                });
            }
        }
    }
}
