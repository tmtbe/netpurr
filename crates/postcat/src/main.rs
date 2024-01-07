#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use chrono::{Datelike, Local, Timelike};
use eframe::icon_data::from_png_bytes;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

use postcat::APP_NAME;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    set_log_config();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(Arc::new(
                from_png_bytes(&include_bytes!("../icon/icon128.png")[..]).unwrap(),
            ))
            .with_inner_size([1000.0, 500.0])
            .with_min_inner_size([800.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(postcat::App::new(cc))
        }),
    )
}

fn set_log_config() {
    let local_time = Local::now();
    // 生成文件名格式
    let file_name = format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}.log",
        local_time.year(),
        local_time.month(),
        local_time.day(),
        local_time.hour(),
        local_time.minute(),
        local_time.second()
    );
    let log_path = dirs::home_dir()
        .unwrap()
        .join(APP_NAME)
        .join("logs")
        .join(file_name);
    let stdout = ConsoleAppender::builder().build();
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(log_path)
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();
    log4rs::init_config(config).expect("init log error");
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
