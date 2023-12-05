use chrono::{DateTime, NaiveDate, Utc};
use eframe::epaint::ahash::HashMap;
use poll_promise::Promise;
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct AppData {
    pub rest_sender: RestSender,
    pub central_request_data_list: CentralRequestDataList,
    pub history_data_list: HistoryDataList,
}
impl AppData {
    pub fn fake(&mut self) {
        self.history_data_list.date_group.insert(
            NaiveDate::default(),
            DateGroupHistoryList {
                history_list: vec![HistoryRestItem {
                    id: Uuid::new_v4().to_string(),
                    record_date: Default::default(),
                    rest: HttpRecord {
                        request: Request {
                            method: Default::default(),
                            url: "http://www.baidu.com".to_string(),
                            params: vec![],
                        },
                        response: Default::default(),
                    },
                }],
            },
        );
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}
impl RestSender {
    pub fn send(&mut self, rest: &mut HttpRecord) -> Promise<ehttp::Result<ehttp::Response>> {
        let (sender, promise) = Promise::new();
        let request = ehttp::Request {
            method: rest.request.method.to_string(),
            url: rest.request.url.to_string(),
            body: vec![],
            headers: Default::default(),
        };
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });
        return promise;
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestDataList {
    pub select_id: Option<String>,
    pub data_list: Vec<CentralRequestItem>,
    pub data_map: HashMap<String, CentralRequestItem>,
}
impl CentralRequestDataList {
    pub fn add_new(&mut self) {
        let crt = CentralRequestItem {
            id: Uuid::new_v4().to_string(),
            rest: Default::default(),
        };
        self.add_crt(crt.clone());
        self.select(crt.id.clone())
    }
    pub fn select(&mut self, id: String) {
        self.select_id = Some(id)
    }
    pub fn add_crt(&mut self, crt: CentralRequestItem) {
        if !self.data_map.contains_key(crt.id.as_str()) {
            self.data_map.insert(crt.id.clone(), crt.clone());
            self.data_list.push(crt.clone())
        }
        self.select(crt.id.clone())
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestItem {
    pub id: String,
    pub rest: HttpRecord,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HistoryDataList {
    pub date_group: HashMap<NaiveDate, DateGroupHistoryList>,
}

impl HistoryDataList {
    pub fn record(&mut self, rest: HttpRecord) {
        let today = chrono::Local::now().naive_local().date();
        if !self.date_group.contains_key(&today) {
            self.date_group.insert(
                today,
                DateGroupHistoryList {
                    history_list: vec![],
                },
            );
        }
        self.date_group
            .get_mut(&today)
            .unwrap()
            .history_list
            .push(HistoryRestItem {
                id: Uuid::new_v4().to_string(),
                record_date: chrono::Local::now().with_timezone(&Utc),
                rest,
            });
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct DateGroupHistoryList {
    pub history_list: Vec<HistoryRestItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HistoryRestItem {
    pub id: String,
    pub record_date: DateTime<Utc>,
    pub rest: HttpRecord,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HttpRecord {
    pub request: Request,
    pub response: Response,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub params: Vec<QueryParam>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Response {
    pub body: Vec<u8>,
}

#[derive(Debug, Display, PartialEq, EnumString, EnumIter, Clone, Eq)]
pub enum Method {
    Post,
    Get,
    Put,
    Patch,
    Delete,
    Copy,
    Head,
    Options,
    Link,
    UnLink,
    Purge,
    Lock,
    UnLock,
    Propfind,
    View,
}

impl Default for Method {
    fn default() -> Self {
        Method::Get
    }
}
