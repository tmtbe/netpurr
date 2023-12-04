use std::cell::RefCell;
use std::rc::Rc;
use egui::Key::S;
use uuid::Uuid;

use crate::events::{MailBox, MailPost};
use crate::events::event::MailEvent;
use crate::models::{CENTRAL_REQUEST_MODELS, Model, ModelStatus};
use crate::models::http::{HttpRecord, Request, Response};

#[derive(PartialEq,Clone)]
pub enum Panel {
    RequestId(String),
}
impl Default for Panel{
    fn default() -> Self {
        Panel::RequestId("".to_string())
    }
}

#[derive(Default)]
pub struct CentralRequestModels {
    pub(crate) open_panel: Panel,
    mail_box: Rc<RefCell<MailBox>>,
    status: Rc<RefCell<ModelStatus<CentralRequestDataList>>>,
    requests: Vec<CentralRequestModel>,
}

#[derive(PartialEq,Eq)]
struct CentralRequestModel {
    id: String,
    rest: HttpRecord,
}

#[derive(Default, Clone)]
pub struct CentralRequestDataList {
    pub data_list: Vec<CentralRequestData>,
}

#[derive(Default,Clone,PartialEq,Eq)]
pub struct CentralRequestData {
    pub id: String,
    pub rest: HttpRecord,
}

impl CentralRequestModels {
    pub fn add_new(&mut self) {
        let cr = CentralRequestModel {
            id: "new".to_string(),
            rest: HttpRecord {
                request: Request {
                    method: "Get".to_string(),
                    url: "Untitled Request".to_string(),
                },
                response: Response {},
            },
        };
        if !self.requests.contains(&cr) {
            self.requests.insert(0, cr);
            self.refresh()
        }
        self.open_panel = Panel::RequestId("new".to_string())
    }
}

impl Model for CentralRequestModels {
    type DataType = CentralRequestDataList;

    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        mail_post.borrow_mut().register(CENTRAL_REQUEST_MODELS.to_string(), self.mail_box.clone())
    }
    fn refresh_data(&mut self) -> Self::DataType {
        return CentralRequestDataList {
            data_list: self.requests.iter().map(|c| {
                CentralRequestData {
                    id:c.id.clone(),
                    rest: c.rest.clone(),
                }
            }).collect()
        };
    }
    fn get_status(&self) -> Rc<RefCell<ModelStatus<Self::DataType>>> {
        self.status.clone()
    }

    fn get_mail_box(&self) -> Rc<RefCell<MailBox>> {
        self.mail_box.clone()
    }

    fn receive(&mut self, mail: MailEvent) {
        match mail {
            MailEvent::AddHttpRecord(record) => {
                let cr = CentralRequestModel {
                    id: record.id.clone(),
                    rest: record.rest,
                };
                if !self.requests.contains(&cr){
                    self.requests.insert(0,cr)
                }
                self.open_panel = Panel::RequestId(record.id.clone());
                self.refresh()
            }
            _ =>{}
        }
    }
}