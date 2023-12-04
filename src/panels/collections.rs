use std::cell::RefCell;
use std::rc::Rc;
use egui::Ui;

use crate::{MailPost, View};

#[derive(Copy, Clone, PartialEq, Default)]
pub struct CollectionsPanel {}

impl View for CollectionsPanel {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {

    }

    fn render(&mut self, ui: &mut Ui,mail_post: Rc<RefCell<MailPost>>) {}
}