use std::cell::RefCell;
use std::rc::Rc;

use crate::events::{MailBox, MailPost};
use crate::events::event::MailEvent;
use crate::models::{CENTRAL_REQUEST_MODELS, Model, ModelStatus};
use crate::models::http::{HttpRecord, Request, Response};

#[derive(Default)]
pub(crate) struct CentralRequestModels {
    mail_box: Rc<RefCell<MailBox>>,
    status: Rc<RefCell<ModelStatus<CentralRequestDataList>>>,
    requests: Vec<CentralRequestModel>,
}

struct CentralRequestModel {
    rest: HttpRecord,
}

#[derive(Default, Clone)]
pub(crate) struct CentralRequestDataList {
    pub(crate) data_list: Vec<CentralRequestData>,
}

#[derive(Clone)]
pub(crate) struct CentralRequestData {
    pub(crate) rest: HttpRecord,
}

impl CentralRequestModels {
    pub(crate) fn add_new(&mut self) {
        self.requests.insert(0, CentralRequestModel {
            rest: HttpRecord {
                request: Request {
                    method: "Get".to_string(),
                    url: "Untitled Request".to_string(),
                },
                response: Response {},
            },
        });
        self.refresh()
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
        println!("{:?}", mail)
    }
}