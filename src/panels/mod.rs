use std::cell::RefCell;
use std::rc::Rc;

use crate::events::MailPost;

pub mod history;
pub mod left;
pub mod collections;
pub mod central;


pub const HORIZONTAL_GAP: f32 = 8.0;
pub const VERTICAL_GAP: f32 = 8.0;

pub(crate) trait View {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>);
    fn render(&mut self, ui: &mut egui::Ui, mail_post: Rc<RefCell<MailPost>>);
}