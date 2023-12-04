use std::cell::RefCell;
use std::rc::Rc;

use egui::{CollapsingHeader, RichText, Ui};

use crate::events::event::MailEvent;
use crate::events::MailPost;
use crate::models::{CENTRAL_REQUEST_MODELS, Model};
use crate::models::history_model::{DateGroupHistoryData, HistoryData, HistoryModels};
use crate::panels::View;

#[derive(Default)]
pub struct HistoryPanel {
    model: HistoryModels,
}

impl View for HistoryPanel {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        self.model.init(mail_post)
    }

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
        self.model.get_data().render(ui, mail_post)
    }
}


impl View for HistoryData {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {}

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
        for mut dateHistoryData in self.date_group_list.clone() {
            CollapsingHeader::new(dateHistoryData.date.clone())
                .default_open(false)
                .show(ui, |ui| dateHistoryData.render(ui, mail_post.clone()));
        }
    }
}

impl View for DateGroupHistoryData {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {}

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {
        for history in self.history_list.clone() {
            if ui.button(RichText::new(history.rest.clone().request.method + &*history.rest.clone().request.url)
                .color(ui.visuals().warn_fg_color)).clicked() {
                mail_post.borrow_mut().send(CENTRAL_REQUEST_MODELS.to_string(), MailEvent::AddHttpRecord(history.clone()))
            }
        }
    }
}
