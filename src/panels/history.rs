use std::cell::RefCell;
use std::rc::Rc;
use egui::{CollapsingHeader, RichText, Ui};

use crate::models::history::{DateGroupHistoryData, HistoryData, HistoryModels};
use crate::{MailPost, Model, View};

#[derive(Default)]
pub struct HistoryPanel {
    model:HistoryModels,
}

impl View for HistoryPanel {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        self.model.init(mail_post)
    }

    fn render(&mut self, ui: &mut Ui,mail_post: Rc<RefCell<MailPost>>) {
        self.model.get_data().render(ui,mail_post)
    }
}


impl View for HistoryData {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {

    }

    fn render(&mut self, ui: &mut Ui,mail_post: Rc<RefCell<MailPost>>) {
        for mut dateHistoryData in self.date_group_list.clone() {
            CollapsingHeader::new(dateHistoryData.date.clone())
                .default_open(false)
                .show(ui, |ui| dateHistoryData.render(ui,mail_post.clone()));
        }
    }
}

impl View for DateGroupHistoryData {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {

    }

    fn render(&mut self, ui: &mut Ui,mail_post: Rc<RefCell<MailPost>>) {
        for history in self.history_list.clone() {
            if ui.button(RichText::new(history.rest.request.method+ &*history.rest.request.url)
                .color(ui.visuals().warn_fg_color)).clicked(){
                mail_post.borrow_mut().send("test".to_string(),"hello".to_string())
            }
        }
    }
}
