use std::cell::RefCell;
use std::rc::Rc;

use crate::events::MailPost;

pub mod history_panel;
pub mod left_panel;
pub mod collections_panel;
pub mod central_panel;


pub const HORIZONTAL_GAP: f32 = 8.0;
pub const VERTICAL_GAP: f32 = 8.0;

pub trait View {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>);
    fn render(&mut self, ui: &mut egui::Ui, mail_post: Rc<RefCell<MailPost>>);
}