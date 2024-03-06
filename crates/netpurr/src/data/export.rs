use serde::{Deserialize, Serialize};

use netpurr_core::data::collections::Collection;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Export {
    pub openapi: Option<String>,
    pub info: Option<PostmanInfo>,
    pub export_type: ExportType,
    pub collection: Option<Collection>,
}
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PostmanInfo {
    pub _postman_id: Option<String>,
    pub name: Option<String>,
    pub schema: Option<String>,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ExportType {
    None,
    Collection,
    Request,
    Environment,
}

impl Default for ExportType {
    fn default() -> Self {
        ExportType::None
    }
}
