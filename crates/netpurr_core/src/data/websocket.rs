use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::data::http::{HttpRecord, Request, RequestSchema};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WebSocketRecord {
    pub http_record: HttpRecord,
    pub select_message_type: MessageType,
    pub retain_content: String,
    pub history_send_messages: Vec<(MessageType, String)>,
    #[serde(skip)]
    pub session: Option<WebSocketSession>,
}

impl Default for WebSocketRecord {
    fn default() -> Self {
        WebSocketRecord {
            http_record: HttpRecord {
                name: "".to_string(),
                desc: "".to_string(),
                request: Request {
                    method: Default::default(),
                    schema: RequestSchema::WS,
                    raw_url: "".to_string(),
                    base_url: "".to_string(),
                    path_variables: vec![],
                    params: vec![],
                    headers: vec![],
                    body: Default::default(),
                    auth: Default::default(),
                },
                response: Default::default(),
                status: Default::default(),
                pre_request_script: "".to_string(),
                test_script: "".to_string(),
            },
            select_message_type: Default::default(),
            retain_content: "".to_string(),
            history_send_messages: vec![],
            session: None,
        }
    }
}

impl WebSocketRecord {
    pub fn compute_signature(&self) -> String {
        format!(
            "HttpRecord:{} History:{}",
            self.http_record.compute_signature(),
            self.history_send_messages.len()
        )
    }
}
#[derive(Default, Clone, Debug)]
pub struct SessionState {
    status: WebSocketStatus,
    messages: Messages,
    events: Vec<WebSocketStatus>,
}

#[derive(Default, Clone, Debug)]
pub struct Messages {
    inner: Vec<WebSocketMessage>,
}

impl DerefMut for Messages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Deref for Messages {
    type Target = Vec<WebSocketMessage>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub enum WebSocketMessage {
    Send(DateTime<Local>, tokio_tungstenite::tungstenite::Message),
    Receive(DateTime<Local>, tokio_tungstenite::tungstenite::Message),
}

#[derive(Clone, Debug, PartialEq)]
pub enum WebSocketStatus {
    Connect,
    Connecting,
    Disconnect,
    ConnectError(String),
    SendError(String),
    SendSuccess,
}

impl Default for WebSocketStatus {
    fn default() -> Self {
        WebSocketStatus::Disconnect
    }
}

#[derive(Clone, Debug)]
pub struct WebSocketSession {
    pub state: Arc<Mutex<SessionState>>,
    pub url: Url,
    pub sender: Sender<tokio_tungstenite::tungstenite::Message>,
}

impl WebSocketSession {
    pub fn add_message(&self, message: WebSocketMessage) {
        self.state.lock().unwrap().messages.push(message.clone());
        if let WebSocketMessage::Send(_, msg) = message {
            self.sender.send(msg);
        }
    }
    pub fn send_message(&self, msg_type: MessageType, msg: String) {
        match msg_type {
            MessageType::Text => {
                self.add_message(WebSocketMessage::Send(Local::now(), Message::Text(msg)));
                self.add_event(WebSocketStatus::SendSuccess)
            }
            MessageType::Binary => {
                let decoded_data = general_purpose::STANDARD.decode(&msg);
                match decoded_data {
                    Ok(data) => self
                        .add_message(WebSocketMessage::Send(Local::now(), Message::Binary(data))),
                    Err(e) => self.add_event(WebSocketStatus::SendError(e.to_string())),
                }
            }
        }
    }

    pub fn disconnect(&self) {
        self.set_status(WebSocketStatus::Disconnect)
    }
    pub fn get_status(&self) -> WebSocketStatus {
        self.state.lock().unwrap().status.clone()
    }
    pub fn set_status(&self, status: WebSocketStatus) {
        self.state.lock().unwrap().status = status.clone();
        self.add_event(status.clone())
    }

    pub fn next_event(&self) -> Option<WebSocketStatus> {
        self.state.lock().unwrap().events.pop()
    }

    pub fn add_event(&self, web_socket_status: WebSocketStatus) {
        self.state
            .lock()
            .unwrap()
            .events
            .insert(0, web_socket_status)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, EnumIter, EnumString, Display)]
pub enum MessageType {
    Text,
    Binary,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Text
    }
}
