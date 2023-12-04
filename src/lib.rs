#![warn(clippy::all, rust_2018_idioms)]

use std::hash::Hash;

pub use app::App;

mod app;
mod panels;
mod models;
mod events;
