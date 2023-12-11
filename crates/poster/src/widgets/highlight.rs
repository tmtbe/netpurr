use eframe::epaint::text::{LayoutJob, TextFormat};
use egui::{Color32, Ui};
use regex::Regex;

pub fn highlight_impl(mut text: &str, ui: &Ui) -> LayoutJob {
    let re = Regex::new(r"\{\{.*?}}").unwrap();
    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(10.0);
    let format = TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 100, 100));
    let normal_format = TextFormat::simple(font_id.clone(), ui.visuals().text_color().clone());
    let mut start = 0;
    for x in re.find_iter(text) {
        job.append(&text[start..x.range().start], 0.0, normal_format.clone());
        job.append(&text[x.range().start..x.range().end], 0.0, format.clone());
        start = x.range().end
    }
    job.append(&text[start..text.len()], 0.0, normal_format.clone());
    job
}
