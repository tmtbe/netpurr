#![warn(clippy::all, rust_2018_idioms)]

use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use egui::ahash::HashMap;
pub use app::App;

mod app;
mod panels;
mod models;

const HORIZONTAL_GAP: f32 = 8.0;
const VERTICAL_GAP: f32 = 8.0;

pub(crate) trait View {
    fn init(&mut self,mail_post: Rc<RefCell<MailPost>>);
    fn render(&mut self, ui: &mut egui::Ui,mail_post: Rc<RefCell<MailPost>>);
}
pub trait Model {
    type DataType:Clone;
    fn init(&mut self,mail_post: Rc<RefCell<MailPost>>);
    fn get_data(&mut self) ->Self::DataType{
        //处理邮箱
        let mail = self.get_mail_box().borrow_mut().mails.pop();
        if mail.is_some(){
            self.receive(mail.unwrap())
        }
        //处理model生成data
        if !self.get_status().borrow().cached{
            let data = self.refresh_data();
            self.get_status().borrow_mut().cache = data;
            self.get_status().borrow_mut().cached = true;
        }
        return self.get_status().borrow().cache.clone()
    }

    fn refresh_data(&mut self)->Self::DataType;

    fn get_status(&self)->Rc<RefCell<ModelStatus<Self::DataType>>>;
    fn get_mail_box(&self)->Rc<RefCell<MailBox>>;
    fn receive(&mut self,mail:String){

    }
    fn refresh(&self){
        self.get_status().borrow_mut().cached = false
    }
}
#[derive(Default)]
pub struct ModelStatus<T:Clone>{
    cached:bool,
    cache:T,
}
#[derive(Default)]
pub struct MailBox{
    mails:Vec<String>
}
#[derive(Default)]
pub struct MailPost {
    nodes:HashMap<String,Rc<RefCell<MailBox>>>
}
impl MailPost {
    fn register(&mut self,name:String,mail_box: Rc<RefCell<MailBox>>){
        self.nodes.insert(name,mail_box);
    }
    fn send(&mut self,who:String,mail:String){
        let who = self.nodes.get(&*who);
        if who.is_some(){
            who.unwrap().borrow_mut().mails.push(mail)
        }
    }
}