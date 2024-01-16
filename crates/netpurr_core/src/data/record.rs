use serde::{Deserialize, Serialize};

use crate::data::http::HttpRecord;
use crate::data::websocket::WebSocketRecord;

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum Record {
    Rest(HttpRecord),
    WebSocket(WebSocketRecord),
}

impl Record {
    pub fn set_pre_request_script(&mut self, script: String) {
        match self {
            Record::Rest(rest) => rest.pre_request_script = script,
            Record::WebSocket(websocket) => websocket.pre_request_script = script,
        }
    }
    pub fn set_test_script(&mut self, script: String) {
        match self {
            Record::Rest(rest) => rest.test_script = script,
            Record::WebSocket(websocket) => websocket.test_script = script,
        }
    }
    pub fn pre_request_script(&self) -> String {
        match self {
            Record::Rest(rest) => rest.pre_request_script.clone(),
            Record::WebSocket(websocket) => websocket.pre_request_script.clone(),
        }
    }
    pub fn test_script(&self) -> String {
        match self {
            Record::Rest(rest) => rest.test_script.clone(),
            Record::WebSocket(websocket) => websocket.test_script.clone(),
        }
    }
    pub fn must_get_rest(&self) -> &HttpRecord {
        match self {
            Record::Rest(rest) => rest,
            Record::WebSocket(_) => panic!("not rest"),
        }
    }
    pub fn must_get_mut_rest(&mut self) -> &mut HttpRecord {
        match self {
            Record::Rest(rest) => rest,
            Record::WebSocket(_) => panic!("not rest"),
        }
    }
    pub fn must_get_websocket(&self) -> &WebSocketRecord {
        match self {
            Record::Rest(_) => panic!("not websocket"),
            Record::WebSocket(websocket) => websocket,
        }
    }
    pub fn must_get_mut_websocket(&mut self) -> &mut WebSocketRecord {
        match self {
            Record::Rest(_) => panic!("not websocket"),
            Record::WebSocket(websocket) => websocket,
        }
    }
    pub fn desc(&self) -> String {
        match self {
            Record::Rest(rest) => rest.desc.clone(),
            Record::WebSocket(websocket) => websocket.desc.clone(),
        }
    }
    pub fn set_desc(&mut self, desc: String) {
        match self {
            Record::Rest(rest) => rest.desc = desc,
            Record::WebSocket(websocket) => websocket.desc = desc,
        }
    }
    pub fn name(&self) -> String {
        match self {
            Record::Rest(rest) => rest.name.clone(),
            Record::WebSocket(websocket) => websocket.name.clone(),
        }
    }

    pub fn method(&self) -> String {
        match self {
            Record::Rest(rest) => rest.request.method.to_string(),
            Record::WebSocket(websocket) => "Ws".to_string(),
        }
    }

    pub fn base_url(&self) -> String {
        match self {
            Record::Rest(rest) => rest.request.base_url.to_string(),
            Record::WebSocket(websocket) => todo!(),
        }
    }
    pub fn set_name(&mut self, name: String) {
        match self {
            Record::Rest(rest) => rest.name = name,
            Record::WebSocket(websocket) => websocket.name = name,
        }
    }

    pub fn get_tab_name(&self) -> String {
        match self {
            Record::Rest(rest) => {
                if rest.name != "" {
                    rest.name.clone()
                } else {
                    if rest.request.base_url == "" {
                        "Untitled Request".to_string()
                    } else {
                        rest.request.base_url.clone()
                    }
                }
            }
            Record::WebSocket(websocket) => {
                todo!()
            }
        }
    }

    pub fn compute_signature(&self) -> String {
        match self {
            Record::Rest(rest) => {
                format!(
                    "Request:[{}] TestScript:[{}] PreRequestScript:[{}]",
                    rest.request.compute_signature(),
                    rest.test_script.clone(),
                    rest.pre_request_script.clone()
                )
            }
            Record::WebSocket(websocket) => {
                todo!()
            }
        }
    }
}

impl Default for Record {
    fn default() -> Self {
        Record::Rest(HttpRecord::default())
    }
}
