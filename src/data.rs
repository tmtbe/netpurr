use chrono::{DateTime, Utc};
use eframe::epaint::ahash::HashMap;
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct AppData {
    pub central_request_data_list: CentralRequestDataList,
    pub history_data_list: HistoryDataList,
}

impl AppData{
    pub fn fake(&mut self){
        self.history_data_list.date_group_list.push(DateGroupHistoryData{
            date: "Nov 17".to_string(),
            history_list: vec![
                HistoryRestItem{
                    id: Uuid::new_v4().to_string(),
                    record_date: Default::default(),
                    rest: HttpRecord{
                        request: Request{
                            method: Default::default(),
                            url: "/xxx/xxx".to_string(),
                            params: vec![],
                        },
                        response: Default::default(),
                    },
                }
            ],
        })
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestDataList {
    pub select_id: Option<String>,
    pub data_list: Vec<CentralRequestItem>,
    pub data_map: HashMap<String, CentralRequestItem>,
}
impl CentralRequestDataList{
    pub fn add_new(&mut self){
        let crt = CentralRequestItem{
            id: Uuid::new_v4().to_string(),
            rest: Default::default(),
        };
        self.add_crt(crt.clone());
        self.select(crt.id.clone())
    }
    pub fn select(&mut self,id:String){
        self.select_id = Some(id)
    }
    pub fn add_crt(&mut self,crt:CentralRequestItem){
        if !self.data_map.contains_key(crt.id.as_str()){
            self.data_map.insert(crt.id.clone(),crt.clone());
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
    pub date_group_list: Vec<DateGroupHistoryData>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct DateGroupHistoryData {
    pub date: String,
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
pub struct Response {}

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
        Method::Post
    }
}