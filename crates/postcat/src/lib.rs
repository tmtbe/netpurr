#![warn(clippy::all, rust_2018_idioms)]

use std::string::ToString;

pub use app::App;

pub const APP_NAME: &str = "Postcat";

mod app;
mod data;
mod import;
mod operation;
mod panels;
mod persistence;
mod script;
mod utils;
mod widgets;
mod windows;
