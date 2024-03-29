use eframe::epaint::FontFamily;
use egui::{Color32, FontId, TextFormat};
use egui::text::LayoutJob;

use netpurr_core::data::test::{TestResult, TestStatus};

use crate::panels::HORIZONTAL_GAP;

#[derive(Default)]
pub struct TestResultPanel {}

impl TestResultPanel {
    pub fn set_and_render(&mut self, ui: &mut egui::Ui, test_result: &TestResult) {
        ui.push_id("test_info", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for test_info in test_result.test_info_list.iter() {
                    ui.horizontal(|ui| {
                        ui.add_space(HORIZONTAL_GAP * 2.0);
                        ui.label(Self::build_status_job(test_info.status.clone()));
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.label(test_info.name.clone());
                            for tar in test_info.results.iter() {
                                ui.horizontal(|ui| {
                                    ui.add_space(HORIZONTAL_GAP * 2.0);
                                    ui.separator();
                                    ui.label(Self::build_status_job(tar.assert_result.clone()));
                                    ui.label(tar.msg.to_string());
                                });
                            }
                        });
                    });
                }
            });
        });
    }

    fn build_status_job(status: TestStatus) -> LayoutJob {
        let mut job = LayoutJob::default();
        let text_format = match status {
            TestStatus::FAIL => TextFormat {
                color: Color32::WHITE,
                background: Color32::DARK_RED,
                font_id: FontId {
                    size: 14.0,
                    family: FontFamily::Monospace,
                },
                ..Default::default()
            },
            _ => TextFormat {
                color: Color32::WHITE,
                background: Color32::DARK_GREEN,
                font_id: FontId {
                    size: 14.0,
                    family: FontFamily::Monospace,
                },
                ..Default::default()
            },
        };
        job.append(status.to_string().as_str(), 0.0, text_format);
        job
    }
}
