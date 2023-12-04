use std::cell::RefCell;
use std::rc::Rc;

use eframe::epaint::ahash::HashMap;

use crate::events::event::MailEvent;

pub mod event;


#[derive(Default)]
pub struct MailBox {
    pub mails: Vec<MailEvent>,
}

#[derive(Default)]
pub struct MailPost {
    nodes: HashMap<String, Rc<RefCell<MailBox>>>,
}

impl MailPost {
    pub fn register(&mut self, name: String, mail_box: Rc<RefCell<MailBox>>) {
        self.nodes.insert(name, mail_box);
    }
    pub fn send(&mut self, who: String, mail: MailEvent) {
        let who = self.nodes.get(&*who);
        if who.is_some() {
            who.unwrap().borrow_mut().mails.push(mail)
        }
    }
}