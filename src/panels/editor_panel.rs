use std::cell::RefCell;
use std::rc::Rc;
use egui::Ui;
use crate::events::MailPost;
use crate::models::central_model::CentralRequestData;
use crate::panels::View;

#[derive(Default)]
pub struct EditorPanel{
    central_request_data:CentralRequestData
}
impl EditorPanel{
    pub(crate) fn set(&mut self, cr:CentralRequestData){
        self.central_request_data = cr
    }
}
impl View for EditorPanel{
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {

    }

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
        ui.vertical(|ui|{
            ui.label(self.central_request_data.rest.request.url.clone());
        });
    }
}