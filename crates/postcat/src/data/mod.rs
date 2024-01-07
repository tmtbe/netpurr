use std::fmt::Display;
use std::io::{Read, Write};
use std::str::FromStr;

use base64::Engine;
use egui::TextBuffer;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::persistence::PersistenceItem;

pub mod auth;
pub mod central_request_data;
pub mod collections;
pub mod config_data;
pub mod cookies_manager;
pub mod environment;
pub mod environment_function;
pub mod export;
pub mod history;
pub mod http;
pub mod logger;
pub mod test;
pub mod workspace;
