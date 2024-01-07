use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use base64::engine::general_purpose;
use base64::Engine;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::auth::Auth;
use crate::data::environment::EnvironmentItemValue;
use crate::data::logger::Logger;

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpRecord {
    pub name: String,
    pub desc: String,
    pub request: Request,
    #[serde(skip)]
    pub response: Response,
    pub status: ResponseStatus,
    pub pre_request_script: String,
    pub test_script: String,
}

impl HttpRecord {
    pub fn pending(&mut self) {
        self.status = ResponseStatus::Pending;
    }
    pub fn ready(&mut self) {
        self.status = ResponseStatus::Ready;
    }
    pub fn none(&mut self) {
        self.status = ResponseStatus::None;
    }
    pub fn error(&mut self) {
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
#[serde(default)]
pub struct Request {
    pub method: Method,
    pub base_url: String,
    pub params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: HttpBody,
    pub auth: Auth,
}

impl Request {
    pub fn compute_signature(&self) -> String {
        let parmas: Vec<String> = self
            .params
            .iter()
            .filter(|q| q.lock_with == LockWith::NoLock)
            .map(|q| q.compute_signature())
            .collect();
        let headers: Vec<String> = self
            .headers
            .iter()
            .filter(|h| h.lock_with == LockWith::NoLock)
            .map(|h| h.compute_signature())
            .collect();
        format!(
            "Method:{} BaseUrl:{} Params:[{}] Headers:[{}] Body:{} Auth:{}",
            self.method,
            self.base_url,
            parmas.join(";"),
            headers.join(";"),
            self.body.compute_signature(),
            self.auth.compute_signature()
        )
    }
    pub fn clear_lock_with(&mut self) {
        self.params.retain(|s| s.lock_with == LockWith::NoLock);
        self.headers.retain(|s| s.lock_with == LockWith::NoLock);
        self.body
            .body_form_data
            .retain(|s| s.lock_with == LockWith::NoLock);
        self.body
            .body_xxx_form
            .retain(|s| s.lock_with == LockWith::NoLock);
    }
    pub fn remove_request_content_type(&mut self) {
        self.headers
            .retain(|h| h.key.to_lowercase() != "content-type" || h.lock_with != LockWith::NoLock);
    }
    pub fn set_request_content_type(&mut self, value: String) {
        let mut find = false;
        for header in self.headers.iter_mut() {
            if header.key.to_lowercase() == "content-type" {
                find = true;
                if !header.value.contains(value.as_str()) {
                    header.value = value.clone();
                }
            }
        }
        if !find {
            self.headers.push(Header {
                key: "content-type".to_string(),
                value,
                desc: "".to_string(),
                enable: true,
                lock_with: LockWith::NoLock,
            });
        }
    }
}

impl HttpRecord {
    pub fn sync(&mut self, envs: BTreeMap<String, EnvironmentItemValue>, parent_auth: Auth) {
        self.request
            .auth
            .build_head(&mut self.request.headers, envs.clone(), parent_auth);
        match self.request.body.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                self.set_request_content_type("multipart/form-data".to_string());
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                self.set_request_content_type("application/x-www-form-urlencoded".to_string());
            }
            BodyType::RAW => match self.request.body.body_raw_type {
                BodyRawType::TEXT => self.set_request_content_type("text/plain".to_string()),
                BodyRawType::JSON => self.set_request_content_type("application/json".to_string()),
                BodyRawType::HTML => self.set_request_content_type("text/html".to_string()),
                BodyRawType::XML => self.set_request_content_type("application/xml".to_string()),
                BodyRawType::JavaScript => {
                    self.set_request_content_type("application/javascript".to_string())
                }
            },
            BodyType::BINARY => {
                let path = Path::new(&self.request.body.body_file);
                let content_type = mime_guess::from_path(path);
                self.set_request_content_type(content_type.first_or_octet_stream().to_string());
            }
        }
    }

    pub fn get_response_content_type(&self) -> Option<Header> {
        self.response
            .headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type")
            .cloned()
    }

    pub fn set_request_content_type(&mut self, value: String) {
        self.request.set_request_content_type(value);
    }
    pub fn remove_request_content_type(&mut self) {
        self.request.remove_request_content_type();
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
#[serde(default)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub lock_with: LockWith,
    pub enable: bool,
}

impl QueryParam {
    pub fn compute_signature(&self) -> String {
        format!(
            "Key:{} Value:{} Desc:{} Enable:{}",
            self.key, self.value, self.desc, self.enable
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LockWith {
    LockWithScript,
    LockWithAuto,
    NoLock,
}

impl Default for LockWith {
    fn default() -> Self {
        LockWith::NoLock
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MultipartData {
    pub data_type: MultipartDataType,
    pub key: String,
    pub value: String,
    pub desc: String,
    pub lock_with: LockWith,
    pub enable: bool,
}

impl MultipartData {
    pub fn compute_signature(&self) -> String {
        format!(
            "Key:{} Value:{} Desc:{} Enable:{}",
            self.key, self.value, self.desc, self.enable
        )
    }
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
#[serde(default)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
    pub lock_with: LockWith,
}

impl Header {
    pub fn new_from_map(headers: &HeaderMap) -> Vec<Header> {
        let mut result = vec![];
        for (key, value) in headers.iter() {
            result.push(Header {
                key: key.to_string(),
                value: value.to_str().unwrap().to_string(),
                desc: "".to_string(),
                enable: true,
                lock_with: LockWith::NoLock,
            })
        }
        result
    }
    pub fn compute_signature(&self) -> String {
        format!(
            "Key:{} Value:{} Desc:{} Enable:{}",
            self.key, self.value, self.desc, self.enable
        )
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Response {
    pub body: Arc<HttpBody>,
    pub headers: Vec<Header>,
    pub status: u16,
    pub status_text: String,
    pub elapsed_time: u128,
    #[serde(skip)]
    pub logger: Logger,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpBody {
    pub base64: String,
    pub size: usize,
    pub body_str: String,
    pub body_file: String,
    pub body_type: BodyType,
    pub body_raw_type: BodyRawType,
    pub body_form_data: Vec<MultipartData>,
    pub body_xxx_form: Vec<MultipartData>,
}

impl HttpBody {
    pub fn compute_signature(&self) -> String {
        let body_form_data: Vec<String> = self
            .body_form_data
            .iter()
            .filter(|b| b.lock_with == LockWith::NoLock)
            .map(|b| b.compute_signature())
            .collect();
        let body_xxx_form: Vec<String> = self
            .body_xxx_form
            .iter()
            .filter(|b| b.lock_with == LockWith::NoLock)
            .map(|b| b.compute_signature())
            .collect();
        format!(
            "BodyStr:{} BodyFile:{} BodyType:{} BodyRawType:{} FormData:[{}] XXXForm:[{}]",
            self.body_str,
            self.body_file,
            self.body_type,
            self.body_raw_type,
            body_form_data.join(";"),
            body_xxx_form.join(";")
        )
    }
    pub fn to_vec(&self) -> Vec<u8> {
        general_purpose::STANDARD.decode(&self.base64).unwrap()
    }
    pub fn get_byte_size(&self) -> String {
        if self.size > 1000000 {
            return (self.size / 1000000).to_string() + " MB";
        } else if self.size > 1000 {
            return (self.size / 1000).to_string() + " KB";
        } else {
            return self.size.to_string() + " B";
        }
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            base64: general_purpose::STANDARD.encode(&bytes).to_string(),
            size: bytes.len(),
            body_str: "".to_string(),
            body_file: "".to_string(),
            body_type: Default::default(),
            body_raw_type: Default::default(),
            body_form_data: vec![],
            body_xxx_form: vec![],
        }
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
