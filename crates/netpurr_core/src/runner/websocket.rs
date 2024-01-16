use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use chrono::Local;
use deno_core::futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::data::websocket::WebSocketStatus::{Connect, ConnectError, SendError};
use crate::data::websocket::{Session, WebSocketMessage};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct WebSocketSender {}

impl WebSocketSender {
    pub fn connect(url: Url) -> Session {
        let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
        let session = Session {
            state: Arc::new(Mutex::new(Default::default())),
            url,
            sender,
        };
        let copy_session = session.clone();
        let _ = poll_promise::Promise::spawn_thread("ws", || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(Self::async_connect(copy_session, receiver))
        });
        session
    }

    async fn async_connect(session: Session, receiver: Receiver<Message>) {
        match connect_async(session.url.clone()).await {
            Ok((ws_stream, _)) => {
                session.state.lock().unwrap().status = Connect;
                let (mut tx, rx) = ws_stream.split();
                let copy_session = session.clone();
                tokio::spawn(async move {
                    let mut incoming = rx.map(Result::unwrap);
                    while let Some(message) = incoming.next().await {
                        if copy_session.state.lock().unwrap().status != Connect {
                            break;
                        }
                        copy_session.add_message(WebSocketMessage::Receive(Local::now(), message))
                    }
                });
                loop {
                    if session.state.lock().unwrap().status != Connect {
                        break;
                    }
                    let message = receiver.recv().unwrap();
                    match tx.send(message).await {
                        Ok(_) => {}
                        Err(e) => {
                            session.state.lock().unwrap().status = SendError(e.to_string());
                            break;
                        }
                    };
                }
            }
            Err(e) => {
                session.state.lock().unwrap().status = ConnectError(e.to_string());
            }
        }
    }
}

#[test]
fn test_connect() {
    let session = WebSocketSender::connect(Url::parse("ws://localhost:3012").unwrap());
    let mut index = 1;
    loop {
        println!("{:?}", session.state.lock().unwrap());
        sleep(Duration::from_secs(2));
        if session.state.lock().unwrap().status == Connect {
            session.send_message(Message::Text("hello".to_string()));
        }
        if index > 5 {
            session.disconnect();
        }
        index = index + 1;
    }
}
