use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

use base64::Engine;
use base64::engine::general_purpose;
use chrono::Local;
use deno_core::futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

use crate::data::http::Request;
use crate::data::websocket::{MessageType, WebSocketMessage, WebSocketSession};
use crate::data::websocket::WebSocketStatus::{Connect, ConnectError, Connecting, SendError};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct WebSocketSender {}

impl WebSocketSender {
    pub fn connect(request: Request) -> WebSocketSession {
        let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
        let mut session = WebSocketSession {
            state: Arc::new(Mutex::new(Default::default())),
            sender,
        };
        session.set_status(Connecting);
        let copy_session = session.clone();
        let _ = poll_promise::Promise::spawn_thread("ws", || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(Self::async_connect(copy_session, request, receiver))
        });
        session
    }
    async fn async_connect<R>(
        mut session: WebSocketSession,
        request: R,
        receiver: Receiver<Message>,
    ) where
        R: IntoClientRequest + Unpin,
    {
        match connect_async(request).await {
            Ok((ws_stream, response)) => {
                session.set_response(response);
                session.set_status(Connect);
                let (mut tx, rx) = ws_stream.split();
                let copy_session = session.clone();
                tokio::spawn(async move {
                    let mut incoming = rx.map(Result::unwrap);
                    while let Some(message) = incoming.next().await {
                        if copy_session.get_status() != Connect {
                            break;
                        }
                        match message {
                            Message::Text(text) => copy_session.add_message(
                                WebSocketMessage::Receive(Local::now(), MessageType::Text, text),
                            ),
                            Message::Binary(b) => {
                                copy_session.add_message(WebSocketMessage::Receive(
                                    Local::now(),
                                    MessageType::Binary,
                                    general_purpose::STANDARD.encode(b),
                                ))
                            }
                            _ => {}
                        }
                    }
                });
                loop {
                    if session.get_status() != Connect {
                        break;
                    }
                    let message = receiver.recv().unwrap();
                    match tx.send(message).await {
                        Ok(_) => {}
                        Err(e) => {
                            session.set_status(SendError(e.to_string()));
                            break;
                        }
                    };
                }
            }
            Err(e) => {
                session.set_status(ConnectError(e.to_string()));
            }
        }
    }
}
