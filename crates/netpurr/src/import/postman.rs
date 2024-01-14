use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::data::auth::{Auth, AuthType};
use crate::data::collections::{Collection, CollectionFolder};
use crate::data::environment::{EnvironmentConfig, EnvironmentItem, EnvironmentValueType};
use crate::data::http::{
    BodyRawType, Header, HttpBody, HttpRecord, Method, MultipartData, MultipartDataType,
    PathVariables, QueryParam, Request, RequestSchema,
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Postman {
    info: PostmanInfo,
    item: Vec<PostmanItemGroup>,
    event: Vec<PostmanEvent>,
    variable: Vec<PostmanVariable>,
    auth: PostmanAuth,
}

impl Postman {
    pub fn try_import(json: String) -> serde_json::Result<Postman> {
        serde_json::from_str(json.as_str())
    }

    pub fn to_collection(&self) -> anyhow::Result<Collection> {
        if self.info.name.is_empty() {
            Err(anyhow!("not postman"))
        } else {
            let collection = Collection {
                envs: EnvironmentConfig {
                    items: self.variable.iter().map(|v| v.to()).collect(),
                },
                folder: Rc::new(RefCell::new(CollectionFolder {
                    name: self.info.name.clone(),
                    parent_path: ".".to_string(),
                    desc: self.info.description.clone(),
                    auth: self.auth.to(),
                    is_root: true,
                    requests: PostmanItemGroup::gen_requests(self.item.clone()),
                    folders: PostmanItemGroup::gen_folders(self.item.clone()),
                    pre_request_script: "".to_string(),
                    test_script: "".to_string(),
                })),
            };
            Ok(collection)
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanInfo {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanItemGroup {
    name: String,
    description: String,
    variable: Vec<PostmanVariable>,
    item: Vec<PostmanItemGroup>,
    request: PostmanRequest,
    event: Vec<PostmanEvent>,
    auth: PostmanAuth,
}

impl PostmanItemGroup {
    pub fn gen_requests(pgs: Vec<PostmanItemGroup>) -> BTreeMap<String, HttpRecord> {
        let http_records: Vec<HttpRecord> = pgs
            .iter()
            .filter(|p| p.item.is_empty())
            .map(|p| HttpRecord {
                name: p.name.to_string(),
                desc: p.description.to_string(),
                request: Request {
                    method: Method::from_str(p.request.method.to_uppercase().as_str())
                        .unwrap_or_default(),
                    schema: RequestSchema::from_str(p.request.url.protocol.to_uppercase().as_str())
                        .unwrap_or_default(),
                    raw_url: p.request.url.raw.clone(),
                    base_url: "".to_string(),
                    path_variables: p
                        .request
                        .url
                        .variable
                        .iter()
                        .map(|v| PathVariables {
                            key: v.key.clone(),
                            value: v.value.clone(),
                            desc: v.description.clone(),
                        })
                        .collect(),
                    params: p
                        .request
                        .url
                        .query
                        .iter()
                        .map(|q| QueryParam {
                            key: q.key.clone(),
                            value: q.value.clone(),
                            desc: q.description.clone(),
                            lock_with: Default::default(),
                            enable: !q.disabled,
                        })
                        .collect(),
                    headers: p
                        .request
                        .header
                        .iter()
                        .map(|h| Header {
                            key: h.key.clone(),
                            value: h.value.clone(),
                            desc: h.description.clone(),
                            enable: !h.disabled,
                            lock_with: Default::default(),
                        })
                        .collect(),
                    body: p.request.body.to(),
                    auth: p.auth.to(),
                },
                ..Default::default()
            })
            .collect();
        let mut result = BTreeMap::default();
        for http_record in http_records.iter() {
            let mut record_clone = http_record.clone();
            record_clone.request.parse_raw_url();
            result.insert(http_record.name.clone(), record_clone);
        }
        result
    }
    pub fn gen_folders(
        pgs: Vec<PostmanItemGroup>,
    ) -> BTreeMap<String, Rc<RefCell<CollectionFolder>>> {
        let folders: Vec<CollectionFolder> = pgs
            .iter()
            .filter(|p| !p.item.is_empty())
            .map(|p| CollectionFolder {
                name: p.name.clone(),
                parent_path: "".to_string(),
                desc: p.description.clone(),
                auth: p.auth.to(),
                is_root: false,
                requests: PostmanItemGroup::gen_requests(p.item.clone()),
                folders: PostmanItemGroup::gen_folders(p.item.clone()),
                pre_request_script: "".to_string(),
                test_script: "".to_string(),
            })
            .collect();
        let mut result = BTreeMap::default();
        for folder in folders.iter() {
            result.insert(folder.name.clone(), Rc::new(RefCell::new(folder.clone())));
        }
        result
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanVariable {
    key: String,
    value: String,
    name: String,
    description: String,
    disabled: bool,
    system: bool,
}

impl PostmanVariable {
    fn to(&self) -> EnvironmentItem {
        EnvironmentItem {
            enable: !self.disabled,
            key: self.key.clone(),
            value: self.value.clone(),
            desc: self.description.clone(),
            value_type: EnvironmentValueType::String,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanRequest {
    auth: PostmanAuth,
    proxy: PostmanProxy,
    method: String,
    description: String,
    header: Vec<PostmanHeader>,
    body: PostmanBody,
    url: PostmanUrl,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanProxy {
    #[serde(alias = "match")]
    proxy_match: String,
    host: String,
    port: i32,
    tunnel: bool,
    disabled: bool,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanBody {
    mode: String,
    raw: String,
    urlencoded: Vec<PostmanUrlEncodedParameter>,
    formdata: Vec<PostmanFormParameter>,
    file: PostmanFile,
    disabled: bool,
}

impl PostmanBody {
    pub fn to(&self) -> HttpBody {
        let mut http_body = HttpBody::default();
        match self.mode.as_str() {
            "raw" => {
                http_body.body_raw_type = BodyRawType::JSON;
                http_body.body_str = self.raw.clone()
            }
            "urlencoded" => {
                for parameter in self.urlencoded.iter() {
                    http_body.body_xxx_form.push(MultipartData {
                        data_type: MultipartDataType::TEXT,
                        key: parameter.key.clone(),
                        value: parameter.value.clone(),
                        desc: parameter.description.clone(),
                        lock_with: Default::default(),
                        enable: !parameter.disabled,
                    });
                }
            }
            "formdata" => {
                for parameter in self.formdata.iter() {
                    let mut value = parameter.value.clone();
                    if parameter.form_type == "file" {
                        value = parameter.src.clone();
                    }
                    http_body.body_form_data.push(MultipartData {
                        data_type: MultipartDataType::from_str(
                            parameter.form_type.to_uppercase().as_str(),
                        )
                        .unwrap_or_default(),
                        key: parameter.key.clone(),
                        value,
                        desc: parameter.description.clone(),
                        lock_with: Default::default(),
                        enable: !parameter.disabled,
                    })
                }
            }
            "file" => http_body.body_file = self.file.src.clone(),
            _ => {}
        }
        http_body
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanFile {
    src: String,
    content: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanUrlEncodedParameter {
    key: String,
    value: String,
    disabled: bool,
    description: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanFormParameter {
    key: String,
    src: String,
    value: String,
    disabled: bool,
    #[serde(alias = "type")]
    form_type: String,
    #[serde(alias = "contentType")]
    content_type: String,
    description: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanAuth {
    #[serde(alias = "type")]
    auth_type: String,
    apikey: Vec<PostmanAuthAttribute>,
    awsv4: Vec<PostmanAuthAttribute>,
    basic: Vec<PostmanAuthAttribute>,
    bearer: Vec<PostmanAuthAttribute>,
    digest: Vec<PostmanAuthAttribute>,
    edgegrid: Vec<PostmanAuthAttribute>,
    hawk: Vec<PostmanAuthAttribute>,
    ntlm: Vec<PostmanAuthAttribute>,
    oauth1: Vec<PostmanAuthAttribute>,
    oauth2: Vec<PostmanAuthAttribute>,
}

impl PostmanAuth {
    pub fn to(&self) -> Auth {
        let auth_type = match self.auth_type.as_str() {
            "bearer" => AuthType::BearerToken,
            "basic" => AuthType::BasicAuth,
            "noauth" => AuthType::NoAuth,
            _ => AuthType::InheritAuthFromParent,
        };
        let basic_username = self
            .basic
            .iter()
            .find(|a| a.key == "username")
            .cloned()
            .unwrap_or_default()
            .value
            .clone();
        let basic_password = self
            .basic
            .iter()
            .find(|a| a.key == "password")
            .cloned()
            .unwrap_or_default()
            .value
            .clone();
        let bearer_token = self
            .basic
            .iter()
            .find(|a| a.key == "token")
            .cloned()
            .unwrap_or_default()
            .value
            .clone();
        Auth {
            auth_type,
            basic_username,
            basic_password,
            bearer_token,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanAuthAttribute {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanHeader {
    key: String,
    disabled: bool,
    value: String,
    description: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanUrl {
    raw: String,
    protocol: String,
    host: Vec<String>,
    path: Vec<String>,
    query: Vec<PostmanQuery>,
    port: String,
    variable: Vec<PostmanVariable>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanQuery {
    key: String,
    value: String,
    disabled: bool,
    description: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PostmanEvent {
    listen: String,
    disabled: bool,
}
