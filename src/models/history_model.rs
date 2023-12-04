use std::cell::RefCell;
use std::rc::Rc;

use chrono::{DateTime, Utc};

use crate::events::{MailBox, MailPost};
use crate::models::{Model, ModelStatus};
use crate::models::http::{HttpRecord, Request, Response};

#[derive(Default)]
pub struct HistoryModels {
    mail_box: Rc<RefCell<MailBox>>,
    status: Rc<RefCell<ModelStatus<HistoryData>>>,
    models: Vec<HistoryModel>,
}

pub struct HistoryModel {
    id: i32,
    record_date: DateTime<Utc>,
    rest: HttpRecord,
}

#[derive(Default, Clone)]
pub struct HistoryData {
    pub date_group_list: Vec<DateGroupHistoryData>,
}

#[derive(Clone)]
pub struct DateGroupHistoryData {
    pub date: String,
    pub history_list: Vec<HistoryRestData>,
}

#[derive(Clone)]
pub struct HistoryRestData {
    pub id: i32,
    pub record_date: DateTime<Utc>,
    pub rest: HttpRecord,
}

impl Model for HistoryModels {
    type DataType = HistoryData;

    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {}

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
        return HistoryData {
            date_group_list: vec![
                DateGroupHistoryData {
                    date: "November 28".to_string(),
                    history_list: vec![
                        HistoryRestData {
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
                    ],
                }
            ],
        };
    }

    fn get_status(&self) -> Rc<RefCell<ModelStatus<Self::DataType>>> {
        self.status.clone()
    }

    fn get_mail_box(&self) -> Rc<RefCell<MailBox>> {
        self.mail_box.clone()
    }
}