use std::collections::BTreeMap;

use eframe::epaint::text::{LayoutJob, TextFormat};
use egui::{Color32, Ui};
use regex::Regex;

use netpurr_core::data::environment::EnvironmentItemValue;

pub fn highlight_template(
    mut text: &str,
    size: f32,
    ui: &Ui,
    envs: BTreeMap<String, EnvironmentItemValue>,
) -> LayoutJob {
    let re = Regex::new(r"\{\{.*?}}").unwrap();
    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(size);
    let not_found_format;
    let found_format;
    let normal_format = TextFormat::simple(font_id.clone(), ui.visuals().text_color().clone());
    if ui.visuals().dark_mode {
        not_found_format = TextFormat::simple(font_id.clone(), Color32::RED);
        found_format = TextFormat::simple(font_id.clone(), Color32::GREEN);
    } else {
        not_found_format = TextFormat::simple(font_id.clone(), Color32::DARK_RED);
        found_format = TextFormat::simple(font_id.clone(), Color32::BLUE);
    }
    let mut start = 0;
    for x in re.find_iter(text) {
        job.append(&text[start..x.range().start], 0.0, normal_format.clone());
        let key = x.as_str().trim_start_matches("{{").trim_end_matches("}}");
        if envs.contains_key(key) {
            job.append(
                &text[x.range().start..x.range().end],
                0.0,
                found_format.clone(),
            );
        } else {
            job.append(
                &text[x.range().start..x.range().end],
                0.0,
                not_found_format.clone(),
            );
        }
        start = x.range().end
    }
    job.append(&text[start..text.len()], 0.0, normal_format.clone());
    job
}
