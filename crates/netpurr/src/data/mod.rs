use std::fmt::Display;
use std::io::{Read, Write};
use std::str::FromStr;

use base64::Engine;
use egui::TextBuffer;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub mod config_data;
pub mod export;
pub mod workspace;
