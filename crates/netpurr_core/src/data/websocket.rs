use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
use tokio_tungstenite::tungstenite::Message;

use crate::data::http::{Header, HttpRecord, Request, RequestSchema, Response};

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
                operation_id: None,
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
    pub fn connected(&self) -> bool {
        match &self.session {
            None => false,
            Some(session) => match session.get_status() {
                WebSocketStatus::Connect => true,
                WebSocketStatus::Connecting => false,
                WebSocketStatus::Disconnect => false,
                WebSocketStatus::ConnectError(_) => false,
                WebSocketStatus::SendError(_) => false,
                WebSocketStatus::SendSuccess => true,
            },
        }
    }
}
#[derive(Default, Clone, Debug)]
pub struct SessionState {
    status: WebSocketStatus,
    response: Response,
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
    Send(DateTime<Local>, MessageType, String),
    Receive(DateTime<Local>, MessageType, String),
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
    pub sender: Sender<Message>,
}

impl WebSocketSession {
    pub fn get_messages(&self) -> Messages {
        self.state.lock().unwrap().messages.clone()
    }
    pub fn add_message(&self, message: WebSocketMessage) {
        self.state.lock().unwrap().messages.push(message.clone());
        if let WebSocketMessage::Send(_, msg_type, text) = message {
            match msg_type {
                MessageType::Text => {
                    self.sender.send(Message::Text(text));
                }
                MessageType::Binary => match general_purpose::STANDARD.decode(text) {
                    Ok(b) => {
                        self.sender.send(Message::Binary(b));
                    }
                    Err(e) => self.add_event(WebSocketStatus::SendError(e.to_string())),
                },
            }
        }
    }
    pub fn send_message(&self, msg_type: MessageType, msg: String) {
        self.add_message(WebSocketMessage::Send(Local::now(), msg_type, msg));
        self.add_event(WebSocketStatus::SendSuccess)
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
    pub fn get_response(&self) -> Response {
        self.state.lock().unwrap().response.clone()
    }
    pub fn set_response(
        &self,
        response: tokio_tungstenite::tungstenite::handshake::client::Response,
    ) {
        let http_response = Response {
            body: Arc::new(Default::default()),
            headers: response
                .headers()
                .iter()
                .map(|(name, value)| Header {
                    key: name.to_string(),
                    value: value.to_str().unwrap_or_default().to_string(),
                    desc: "".to_string(),
                    enable: true,
                    lock_with: Default::default(),
                })
                .collect(),
            status: response.status().as_u16(),
            status_text: "".to_string(),
            elapsed_time: 0,
            logger: Default::default(),
        };
        self.state.lock().unwrap().response = http_response;
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
