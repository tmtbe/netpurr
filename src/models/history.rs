use std::cell::RefCell;
use std::rc::Rc;
use chrono::{DateTime, Utc};
use crate::{MailBox, MailPost, Model, ModelStatus};
use crate::models::http::{HttpRecord, Request, Response};

#[derive(Default)]
pub(crate) struct HistoryModels {
    mail_box:Rc<RefCell<MailBox>>,
    status: Rc<RefCell<ModelStatus<HistoryData>>>,
    models: Vec<HistoryModel>,
}

pub(crate) struct HistoryModel {
    id: i32,
    record_date: DateTime<Utc>,
    rest: HttpRecord,
}

#[derive(Default,Clone)]
pub(crate) struct HistoryData {
    pub(crate) date_group_list: Vec<DateGroupHistoryData>,
}

#[derive(Clone)]
pub(crate) struct DateGroupHistoryData {
    pub(crate) date: String,
    pub(crate) history_list: Vec<HistoryRestData>,
}
#[derive(Clone)]
pub(crate) struct HistoryRestData {
    pub(crate) id: i32,
    pub(crate) record_date: DateTime<Utc>,
    pub(crate) rest: HttpRecord,
}
impl Model for HistoryModels {
    type DataType = HistoryData;

    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {

    }

    fn refresh_data(&mut self) -> Self::DataType {
        self.models = vec![
            HistoryModel {
                id: 0,
                record_date: Default::default(),
                rest: HttpRecord {
                    request: Request {
                        method: "Post".to_string(),
                        url: "{{xxx}}/execute".to_string(),
                    },
                    response: Response {},
                },
            }
        ];
        return HistoryData{
            date_group_list: vec![
                DateGroupHistoryData{
                    date: "November 28".to_string(),
                    history_list: vec![
                        HistoryRestData{
                            id: 0,
                            record_date: Default::default(),
                            rest: HttpRecord {
                                request: Request {
                                    method: "Post".to_string(),
                                    url: "{{xxx}}/execute".to_string(),
                                },
                                response: Response {

                                } },
                        }
                    ],
                }
            ],
        }
    }

    fn get_status(&self) -> Rc<RefCell<ModelStatus<Self::DataType>>> {
        self.status.clone()
    }

    fn get_mail_box(&self) -> Rc<RefCell<MailBox>> {
        self.mail_box.clone()
    }
}