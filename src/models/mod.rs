use std::cell::RefCell;
use std::rc::Rc;
use std::string::ToString;

use crate::events::{MailBox, MailPost};
use crate::events::event::MailEvent;

pub mod history_model;
pub mod http;
pub mod central_model;

pub const HISTORY_MODELS: &str = "HISTORY_MODELS";
pub const CENTRAL_REQUEST_MODELS: &str = "CENTRAL_REQUEST_MODELS";


#[derive(Default)]
pub struct ModelStatus<T: Clone> {
    cached: bool,
    cache: T,
}

pub trait Model {
    type DataType: Clone;
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>);
    fn get_data(&mut self) -> Self::DataType {
        //处理邮箱
        let mail = self.get_mail_box().borrow_mut().mails.pop();
        if mail.is_some() {
            self.receive(mail.unwrap())
        }
        //处理model生成data
        if !self.get_status().borrow().cached {
            let data = self.refresh_data();
            self.get_status().borrow_mut().cache = data;
            self.get_status().borrow_mut().cached = true;
        }
        return self.get_status().borrow().cache.clone();
    }

    fn refresh_data(&mut self) -> Self::DataType;

    fn get_status(&self) -> Rc<RefCell<ModelStatus<Self::DataType>>>;
    fn get_mail_box(&self) -> Rc<RefCell<MailBox>>;
    fn receive(&mut self, mail: MailEvent) {}
    fn refresh(&self) {
        self.get_status().borrow_mut().cached = false
    }
}