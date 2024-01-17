use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use base64::engine::general_purpose;
use base64::Engine;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::error::UrlError;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::Error;

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
    pub fn compute_signature(&self) -> String {
        format!(
            "Request:[{}] TestScript:[{}] PreRequestScript:[{}]",
            self.request.compute_signature(),
            self.test_script.clone(),
            self.pre_request_script.clone()
        )
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
    pub schema: RequestSchema,
    pub raw_url: String,
    pub base_url: String,
    pub path_variables: Vec<PathVariables>,
    pub params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: HttpBody,
    pub auth: Auth,
}

impl IntoClientRequest for Request {
    fn into_client_request(
        self,
    ) -> tokio_tungstenite::tungstenite::Result<
        tokio_tungstenite::tungstenite::handshake::client::Request,
    > {
        let uri = self.raw_url.as_str().parse::<Uri>()?;
        let authority = uri
            .authority()
            .ok_or(Error::Url(UrlError::NoHostName))?
            .as_str();
        let host = authority
            .find('@')
            .map(|idx| authority.split_at(idx + 1).1)
            .unwrap_or_else(|| authority);

        if host.is_empty() {
            return Err(Error::Url(UrlError::EmptyHostName));
        }

        let req = tokio_tungstenite::tungstenite::handshake::client::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .uri(uri)
            .body(())?;
        Ok(req)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Display, EnumString)]
pub enum RequestSchema {
    HTTP,
    HTTPS,
    WS,
    WSS,
}

impl Default for RequestSchema {
    fn default() -> Self {
        RequestSchema::HTTP
    }
}
impl Request {
    pub fn fix_base_url(&mut self) {
        let base_list: Vec<&str> = self.base_url.split("://").collect();
        if base_list.len() >= 2 {
            self.base_url = base_list[1].to_string();
        }
    }
    pub fn sync_header(&mut self, envs: BTreeMap<String, EnvironmentItemValue>, parent_auth: Auth) {
        // build auto header
        self.auth
            .build_head(&mut self.headers, envs.clone(), parent_auth);
        match self.body.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                self.set_request_content_type("multipart/form-data".to_string());
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                self.set_request_content_type("application/x-www-form-urlencoded".to_string());
            }
            BodyType::RAW => match self.body.body_raw_type {
                BodyRawType::TEXT => self.set_request_content_type("text/plain".to_string()),
                BodyRawType::JSON => self.set_request_content_type("application/json".to_string()),
                BodyRawType::HTML => self.set_request_content_type("text/html".to_string()),
                BodyRawType::XML => self.set_request_content_type("application/xml".to_string()),
                BodyRawType::JavaScript => {
                    self.set_request_content_type("application/javascript".to_string())
                }
            },
            BodyType::BINARY => {
                let path = Path::new(&self.body.body_file);
                let content_type = mime_guess::from_path(path);
                self.set_request_content_type(content_type.first_or_octet_stream().to_string());
            }
        }
    }
    pub fn get_url_with_schema(&self) -> String {
        format!(
            "{}://{}",
            self.schema.to_string().to_lowercase(),
            self.base_url
        )
    }

    pub fn get_path_variable_keys(&self) -> Vec<String> {
        let mut keys = vec![];
        for url_part in self.base_url.split("/") {
            if url_part.starts_with(":") {
                keys.push(url_part[1..].to_string());
            }
        }
        keys
    }

    pub fn build_raw_url(&mut self) {
        let mut params = vec![];
        for q in self.params.iter().filter(|q| q.enable) {
            params.push(format!("{}={}", q.key, q.value))
        }
        if !params.is_empty() {
            self.raw_url = format!("{}?{}", self.get_url_with_schema(), params.join("&"))
        } else {
            self.raw_url = self.get_url_with_schema();
        }
    }
    pub fn parse_raw_url(&mut self) {
        let raw_url_split: Vec<&str> = self.raw_url.splitn(2, "://").collect();
        let mut schema_str = "http";
        let mut params_url = "";
        if raw_url_split.len() >= 2 {
            schema_str = raw_url_split[0];
            params_url = raw_url_split[1];
        } else {
            params_url = self.raw_url.as_str();
        }
        self.schema =
            RequestSchema::from_str(schema_str.to_uppercase().as_str()).unwrap_or_default();
        let params_url_split: Vec<&str> = params_url.splitn(2, "?").collect();
        self.base_url = params_url_split[0].to_string();
        params_url_split.get(1).map(|params| {
            self.params.retain(|q| !q.enable);
            let mut retain_params: Vec<QueryParam> = self.params.clone();
            self.params.clear();
            for pair_str in params.split("&") {
                let pair_list: Vec<&str> = pair_str.splitn(2, "=").collect();
                if pair_list.len() == 2 {
                    let key = pair_list[0];
                    let value = pair_list[1];
                    self.params.push(QueryParam {
                        key: key.to_string(),
                        value: value.to_string(),
                        desc: "".to_string(),
                        lock_with: Default::default(),
                        enable: true,
                    })
                }
            }
            retain_params.retain(|rp| self.params.iter().find(|p| p.key == rp.key).is_none());
            self.params.append(&mut retain_params);
        });
        let path_variables = self.get_path_variable_keys();
        for path_variable in path_variables.iter() {
            if self
                .path_variables
                .iter()
                .find(|p| p.key == path_variable.clone())
                .is_none()
            {
                self.path_variables.push(PathVariables {
                    key: path_variable.to_string(),
                    value: "".to_string(),
                    desc: "".to_string(),
                })
            }
        }
        self.path_variables
            .retain(|p| path_variables.contains(&p.key));
    }
    pub fn compute_signature(&self) -> String {
        let path_variables: Vec<String> = self
            .path_variables
            .iter()
            .map(|p| p.compute_signature())
            .collect();
        let params: Vec<String> = self
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
            "Schema:{} Method:{} BaseUrl:{} PathVariables:[{}] Params:[{}] Headers:[{}] Body:{} Auth:{}",
            self.schema,
            self.method,
            self.base_url,
            path_variables.join(";"),
            params.join(";"),
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
    pub fn sync_everytime(
        &mut self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        parent_auth: Auth,
    ) {
        self.request.fix_base_url();
        self.request.sync_header(envs, parent_auth);
    }
    pub fn build_raw_url(&mut self) {
        self.request.build_raw_url();
    }
    pub fn sync_raw_url(&mut self) {
        self.request.parse_raw_url();
    }

    pub fn prepare_send(
        &mut self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        parent_auth: Auth,
    ) {
        self.request.clear_lock_with();
        self.sync_everytime(envs, parent_auth);
        self.sync_raw_url();
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

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PathVariables {
    pub key: String,
    pub value: String,
    pub desc: String,
}

impl PathVariables {
    pub fn compute_signature(&self) -> String {
        format!("Key:{} Value:{} Desc:{}", self.key, self.value, self.desc)
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
    FILE,
    TEXT,
}

impl Default for MultipartDataType {
    fn default() -> Self {
        MultipartDataType::TEXT
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
                value: value.to_str().unwrap_or("").to_string(),
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
        general_purpose::STANDARD
            .decode(&self.base64)
            .unwrap_or_default()
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
