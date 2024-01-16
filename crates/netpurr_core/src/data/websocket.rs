use serde::{Deserialize, Serialize};

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WebSocketRecord {
    pub name: String,
    pub desc: String,
    pub pre_request_script: String,
    pub test_script: String,
}
