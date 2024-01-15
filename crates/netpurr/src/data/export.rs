use netpurr_core::data::collections::Collection;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Export {
    pub export_type: ExportType,
    pub collection: Option<Collection>,
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
