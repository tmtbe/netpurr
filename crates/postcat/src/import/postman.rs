use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::data::collections::Collection;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Postman {
    info: PostmanInfo,
    item: Vec<PostmanItemGroup>,
    event: Vec<PostmanEvent>,
}

impl Postman {
    pub fn try_import(json: String) -> serde_json::Result<Postman> {
        serde_json::from_str(json.as_str())
    }

    pub fn to_collection(&self) -> anyhow::Result<Collection> {
        if self.info.name.is_empty() {
            Err(anyhow!("not postman"))
        } else {
            todo!()
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
