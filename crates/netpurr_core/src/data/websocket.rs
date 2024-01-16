use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WebSocketRecord {
    pub name: String,
    pub desc: String,
    pub pre_request_script: String,
    pub test_script: String,
    pub url: String,
    pub send_messages: Vec<Message>,
    #[serde(skip)]
    pub session: Option<Session>,
}

#[derive(Default, Clone, Debug)]
pub struct SessionState {
    pub(crate) status: WebSocketStatus,
    messages: Messages,
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
    Disconnect,
    ConnectError(String),
    SendError(String),
}

impl Default for WebSocketStatus {
    fn default() -> Self {
        WebSocketStatus::Disconnect
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub state: Arc<Mutex<SessionState>>,
    pub url: Url,
    pub sender: Sender<tokio_tungstenite::tungstenite::Message>,
}

impl Session {
    pub fn add_message(&self, message: WebSocketMessage) {
        self.state.lock().unwrap().messages.push(message.clone());
        if let WebSocketMessage::Send(_, msg) = message {
            self.sender.send(msg);
        }
    }
    pub fn send_message(&self, message: tokio_tungstenite::tungstenite::Message) {
        self.add_message(WebSocketMessage::Send(Local::now(), message))
    }

    pub fn disconnect(&self) {
        self.state.lock().unwrap().status = WebSocketStatus::Disconnect
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Message {
    Text(String),
    Binary(String),
}
