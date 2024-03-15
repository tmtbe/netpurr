use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::data::collections::Testcase;
use crate::data::http::HttpRecord;
use crate::data::websocket::WebSocketRecord;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Record {
    Rest(HttpRecord),
    WebSocket(WebSocketRecord),
}

impl Record {
    pub fn set_pre_request_script(&mut self, script: String) {
        match self {
            Record::Rest(rest) => rest.pre_request_script = script,
            Record::WebSocket(websocket) => websocket.http_record.pre_request_script = script,
        }
    }
    pub fn set_test_script(&mut self, script: String) {
        match self {
            Record::Rest(rest) => rest.test_script = script,
            Record::WebSocket(websocket) => websocket.http_record.test_script = script,
        }
    }
    pub fn set_testcases(&mut self, testcases: BTreeMap<String, Testcase>) {
        match self {
            Record::Rest(rest) => rest.testcases = testcases,
            Record::WebSocket(websocket) => {}
        }
    }
    pub fn pre_request_script(&self) -> String {
        match self {
            Record::Rest(rest) => rest.pre_request_script.clone(),
            Record::WebSocket(websocket) => websocket.http_record.pre_request_script.clone(),
        }
    }
    pub fn test_script(&self) -> String {
        match self {
            Record::Rest(rest) => rest.test_script.clone(),
            Record::WebSocket(websocket) => websocket.http_record.test_script.clone(),
        }
    }

    pub fn testcase(&self) -> BTreeMap<String, Testcase> {
        match self {
            Record::Rest(rest) => rest.testcases.clone(),
            Record::WebSocket(websocket) => BTreeMap::new(),
        }
    }
    pub fn must_get_rest(&self) -> &HttpRecord {
        match self {
            Record::Rest(rest) => rest,
            Record::WebSocket(websocket) => &websocket.http_record,
        }
    }
    pub fn must_get_mut_rest(&mut self) -> &mut HttpRecord {
        match self {
            Record::Rest(rest) => rest,
            Record::WebSocket(websocket) => &mut websocket.http_record,
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
            Record::WebSocket(websocket) => websocket.http_record.desc.clone(),
        }
    }
    pub fn set_desc(&mut self, desc: String) {
        match self {
            Record::Rest(rest) => rest.desc = desc,
            Record::WebSocket(websocket) => websocket.http_record.desc = desc,
        }
    }
    pub fn name(&self) -> String {
        match self {
            Record::Rest(rest) => rest.name.clone(),
            Record::WebSocket(websocket) => websocket.http_record.name.clone(),
        }
    }

    pub fn method(&self) -> String {
        match self {
            Record::Rest(rest) => rest.request.method.to_string(),
            Record::WebSocket(websocket) => "WS".to_string(),
        }
    }

    pub fn base_url(&self) -> String {
        match self {
            Record::Rest(rest) => rest.request.base_url.to_string(),
            Record::WebSocket(websocket) => websocket.http_record.request.base_url.to_string(),
        }
    }
    pub fn raw_url(&self) -> String {
        match self {
            Record::Rest(rest) => rest.request.raw_url.to_string(),
            Record::WebSocket(websocket) => websocket.http_record.request.raw_url.to_string(),
        }
    }
    pub fn set_name(&mut self, name: String) {
        match self {
            Record::Rest(rest) => rest.name = name,
            Record::WebSocket(websocket) => websocket.http_record.name = name,
        }
    }

    pub fn get_tab_name(&self) -> String {
        if self.name() != "" {
            self.name()
        } else {
            if self.base_url() == "" {
                "Untitled Request".to_string()
            } else {
                self.base_url()
            }
        }
    }

    pub fn compute_signature(&self) -> String {
        match self {
            Record::Rest(rest) => rest.compute_signature(),
            Record::WebSocket(websocket) => websocket.compute_signature(),
        }
    }
}

impl Default for Record {
    fn default() -> Self {
        Record::Rest(HttpRecord::default())
    }
}
