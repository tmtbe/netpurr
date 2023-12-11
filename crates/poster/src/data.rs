use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use chrono::{DateTime, NaiveDate, Utc};
use eframe::epaint::ahash::HashMap;
use egui::TextBuffer;
use poll_promise::Promise;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use urlencoding::encode;
use uuid::Uuid;

use ehttp::multipart::MultipartBuilder;

use crate::save::{Persistence, PersistenceItem};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct AppData {
    pub rest_sender: RestSender,
    pub central_request_data_list: CentralRequestDataList,
    pub history_data_list: HistoryDataList,
    pub environment: Environment,
}

impl AppData {
    pub fn load_all(&mut self) {
        self.history_data_list.load_all();
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    pub select: Option<String>,
    pub data: BTreeMap<String, EnvironmentConfig>,
}

impl Environment {
    pub fn get_variable_hash_map(&self) -> HashMap<String, String> {
        self.select.clone().map_or_else(HashMap::default, |s| {
            self.data
                .get(s.as_str())
                .map_or_else(HashMap::default, |e| {
                    let mut result = HashMap::default();
                    for x in e.items.iter().filter(|i| i.enable) {
                        result.insert(x.key.clone(), x.value.clone());
                    }
                    result
                })
        })
    }
}

impl Default for Environment {
    fn default() -> Self {
        let mut e = Environment {
            select: None,
            data: Default::default(),
        };
        e.data
            .insert("test".to_string(), EnvironmentConfig::default());
        return e;
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct EnvironmentConfig {
    pub items: Vec<EnvironmentItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct EnvironmentItem {
    pub enable: bool,
    pub key: String,
    pub value: String,
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}

impl RestSender {
    pub fn send(
        &mut self,
        rest: &mut HttpRecord,
        envs: HashMap<String, String>,
    ) -> (Promise<ehttp::Result<ehttp::Response>>, Instant) {
        let (sender, promise) = Promise::new();
        if !rest.request.base_url.starts_with("http://")
            && !rest.request.base_url.starts_with("https://")
        {
            rest.request.base_url = "http://".to_string() + rest.request.base_url.as_str();
        }
        match rest.request.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                let mut multipart = MultipartBuilder::new();
                for x in rest.request.body_form_data.iter_mut() {
                    if !x.enable {
                        continue;
                    }
                    match x.data_type {
                        MultipartDataType::File => {
                            let file = PathBuf::from(x.value.as_str());
                            if !file.is_file() {
                                x.enable = false;
                                continue;
                            }
                            multipart = multipart.add_file(x.key.as_str(), file);
                        }
                        MultipartDataType::Text => {
                            multipart = multipart.add_text(
                                x.key.as_str(),
                                self.replace_variable(x.value.clone(), envs.clone())
                                    .as_str(),
                            );
                        }
                    }
                }
                let (content_type, data) = multipart.build();
                rest.set_content_type(content_type);
                rest.request.body = data
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                let body_part: Vec<String> = rest
                    .request
                    .body_xxx_form
                    .iter()
                    .filter(|x| x.enable)
                    .map(|x| MultipartData {
                        data_type: x.data_type.clone(),
                        key: x.key.clone(),
                        value: self.replace_variable(x.value.clone(), envs.clone()),
                        desc: x.desc.clone(),
                        enable: x.enable,
                    })
                    .map(|x| format!("{}={}", encode(x.key.as_str()), encode(x.value.as_str())))
                    .collect();
                rest.request.body = body_part.join("&").as_bytes().to_vec();
            }
            BodyType::RAW => {
                rest.request.body = self
                    .replace_variable(rest.request.body_str.clone(), envs.clone())
                    .as_bytes()
                    .to_vec();
            }
            BodyType::BINARY => {}
        }
        let request = ehttp::Request {
            method: rest.request.method.to_string(),
            url: self.build_url(rest, envs.clone()),
            body: rest.request.body.clone(),
            headers: rest
                .request
                .headers
                .iter()
                .filter(|h| h.enable)
                .map(|h| Header {
                    key: h.key.clone(),
                    value: self.replace_variable(h.value.clone(), envs.clone()),
                    desc: h.desc.clone(),
                    enable: h.enable,
                })
                .map(|h| (h.key.clone(), h.value.clone()))
                .collect(),
        };

        ehttp::fetch(request, move |response| {
            sender.send(response);
        });
        return (promise, Instant::now());
    }
    fn build_url(&self, rest: &HttpRecord, envs: HashMap<String, String>) -> String {
        let url = self.replace_variable(rest.request.base_url.clone(), envs.clone());
        let params: Vec<String> = rest
            .request
            .params
            .iter()
            .filter(|p| p.enable)
            .map(|p| QueryParam {
                key: p.key.clone(),
                value: self.replace_variable(p.value.clone(), envs.clone()),
                desc: p.desc.clone(),
                enable: p.enable,
            })
            .map(|p| format!("{}={}", encode(p.key.as_str()), encode(p.value.as_str())))
            .collect();
        url + "?" + params.join("&").as_str()
    }

    pub fn replace_variable(&self, content: String, envs: HashMap<String, String>) -> String {
        let re = Regex::new(r"\{\{.*?}}").unwrap();
        let mut result = content.clone();
        loop {
            let temp = result.clone();
            let find = re.find_iter(temp.as_str()).next();
            if find.is_some() {
                let key = find
                    .unwrap()
                    .as_str()
                    .trim_start_matches("{{")
                    .trim_end_matches("}}");
                let v = envs.get(key);
                if v.is_some() {
                    result.replace_range(find.unwrap().range(), v.unwrap())
                } else {
                    result.replace_range(find.unwrap().range(), "{UNKNOWN}")
                }
            } else {
                break;
            }
        }
        result
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestDataList {
    pub select_id: Option<String>,
    pub data_list: Vec<CentralRequestItem>,
    pub data_map: HashMap<String, CentralRequestItem>,
}

impl CentralRequestDataList {
    pub fn remove(&mut self, id: String) {
        self.data_map.remove(id.as_str());
        self.data_list
            .clone()
            .iter()
            .enumerate()
            .find(|(_, c)| c.id == id)
            .map(|(index, _)| self.data_list.remove(index));
    }
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
    persistence: Persistence,
    date_group: HashMap<NaiveDate, DateGroupHistoryList>,
}

impl HistoryDataList {
    pub fn load_all(&mut self) {
        for date_dir in self
            .persistence
            .load_list(Path::new("history").to_path_buf())
            .iter()
        {
            if let Some(date) = date_dir.file_name() {
                if let Some(date_name) = date.to_str() {
                    if let Ok(naive_date) = NaiveDate::from_str(date_name) {
                        let mut date_group_history_list = DateGroupHistoryList::default();
                        for item_path in self.persistence.load_list(date_dir.clone()).iter() {
                            if let Ok(history_rest_item) = self.persistence.load(item_path.clone())
                            {
                                date_group_history_list.history_list.push(history_rest_item);
                            }
                        }
                        self.date_group.insert(naive_date, date_group_history_list);
                    }
                }
            }
        }
    }
    pub fn get_group(&self) -> &HashMap<NaiveDate, DateGroupHistoryList> {
        &self.date_group
    }
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
        let hrt = HistoryRestItem {
            id: Uuid::new_v4().to_string(),
            record_date: chrono::Local::now().with_timezone(&Utc),
            rest,
        };
        self.date_group
            .get_mut(&today)
            .unwrap()
            .history_list
            .push(hrt.clone());
        self.persistence.save(
            Path::new("history").join(today.to_string()),
            hrt.id.clone(),
            &hrt,
        );
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct DateGroupHistoryList {
    pub history_list: Vec<HistoryRestItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HistoryRestItem {
    pub id: String,
    pub record_date: DateTime<Utc>,
    pub rest: HttpRecord,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HttpRecord {
    pub request: Request,
    pub response: Response,
    pub status: ResponseStatus,
    pub elapsed_time: Option<u128>,
}

impl HttpRecord {
    pub(crate) fn pending(&mut self) {
        self.status = ResponseStatus::Pending;
    }
    pub(crate) fn ready(&mut self) {
        self.status = ResponseStatus::Ready;
    }
    pub(crate) fn none(&mut self) {
        self.status = ResponseStatus::None;
    }
    pub(crate) fn error(&mut self) {
        self.status = ResponseStatus::Error;
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ResponseStatus {
    None,
    Pending,
    Ready,
    Error,
}

impl Default for ResponseStatus {
    fn default() -> Self {
        ResponseStatus::None
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub base_url: String,
    pub params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
    pub body_str: String,
    pub body_type: BodyType,
    pub body_raw_type: BodyRawType,
    pub body_form_data: Vec<MultipartData>,
    pub body_xxx_form: Vec<MultipartData>,
}

impl HttpRecord {
    pub fn sync(&mut self) {
        match self.request.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                self.request.method = Method::POST;
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                self.request.method = Method::POST;
                self.set_content_type("application/x-www-form-urlencoded".to_string());
            }
            BodyType::RAW => {
                self.request.method = Method::POST;
                match self.request.body_raw_type {
                    BodyRawType::TEXT => self.set_content_type("text/plain".to_string()),
                    BodyRawType::JSON => self.set_content_type("application/json".to_string()),
                    BodyRawType::HTML => self.set_content_type("text/html".to_string()),
                    BodyRawType::XML => self.set_content_type("application/xml".to_string()),
                    BodyRawType::JavaScript => {
                        self.set_content_type("application/javascript".to_string())
                    }
                }
            }
            BodyType::BINARY => {}
        }
    }

    pub fn set_content_type(&mut self, value: String) {
        let mut need_add = false;
        let mut find = false;
        for (index, header) in self.request.headers.clone().iter().enumerate() {
            if header.key == "content-type" {
                find = true;
                if !header.value.contains(value.as_str()) {
                    need_add = true;
                    self.request.headers.remove(index);
                }
            }
        }
        if !find || need_add {
            self.request.headers.push(Header {
                key: "content-type".to_string(),
                value,
                desc: "".to_string(),
                enable: true,
            });
        }
    }
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BodyRawType {
    TEXT,
    JSON,
    HTML,
    XML,
    JavaScript,
}

impl Default for BodyRawType {
    fn default() -> Self {
        BodyRawType::JSON
    }
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BodyType {
    NONE,
    FROM_DATA,
    X_WWW_FROM_URLENCODED,
    RAW,
    BINARY,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::NONE
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MultipartData {
    pub data_type: MultipartDataType,
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, EnumIter, EnumString, Serialize, Deserialize)]
pub enum MultipartDataType {
    File,
    Text,
}

impl Default for MultipartDataType {
    fn default() -> Self {
        MultipartDataType::Text
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

impl Header {
    pub fn new_from_tuple(headers: Vec<(String, String)>) -> Vec<Header> {
        let mut result = vec![];
        for (key, value) in headers {
            result.push(Header {
                key,
                value,
                desc: "".to_string(),
                enable: true,
            })
        }
        result
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Response {
    pub body: Vec<u8>,
    pub headers: Vec<Header>,
    pub url: String,
    pub ok: bool,
    pub status: u16,
    pub status_text: String,
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: String,
    pub max_age: String,
    pub http_only: bool,
    pub secure: bool,
}

impl Response {
    //BAIDUID=67147D03A8E2F75F66619A1CFADFAAF2:FG=1; expires=Thu, 31-Dec-37 23:55:55 GMT; max-age=2147483647; path=/; domain=.baidu.com
    pub fn get_cookies(&self) -> Vec<Cookie> {
        self.headers
            .iter()
            .filter(|h| h.key.starts_with("set-cookie"))
            .map(|h| {
                let mut cookie = Cookie {
                    name: "".to_string(),
                    value: "".to_string(),
                    domain: "".to_string(),
                    path: "".to_string(),
                    expires: "".to_string(),
                    max_age: "".to_string(),
                    http_only: false,
                    secure: false,
                };
                let s = h.value.split(";");
                for x in s {
                    let one: Vec<&str> = x.splitn(2, "=").collect();
                    match one[0].trim() {
                        "expires" => cookie.expires = one[1].to_string(),
                        "path" => cookie.path = one[1].to_string(),
                        "domain" => cookie.domain = one[1].to_string(),
                        "max-age" => cookie.max_age = one[1].to_string(),
                        "secure" => cookie.secure = true,
                        "httponly" => cookie.http_only = true,
                        _ => {
                            cookie.value = one[1].to_string();
                            cookie.name = one[0].to_string()
                        }
                    }
                }
                cookie
            })
            .collect()
    }
}

#[derive(Debug, Display, PartialEq, EnumString, EnumIter, Clone, Eq, Serialize, Deserialize)]
pub enum Method {
    POST,
    GET,
    PUT,
    PATCH,
    DELETE,
    COPY,
    HEAD,
    OPTIONS,
    LINK,
    UNLINK,
    PURGE,
    LOCK,
    UNLOCK,
    PROPFIND,
    VIEW,
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}
