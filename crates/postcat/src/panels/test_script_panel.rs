use std::ops::Add;

use egui::Ui;

use crate::panels::HORIZONTAL_GAP;

#[derive(Default)]
pub struct TestScriptPanel {}

impl TestScriptPanel {
    pub(crate) fn set_and_render(&mut self, ui: &mut Ui, mut script: String, id: String) -> String {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "js");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        ui.horizontal(|ui| {
            egui::SidePanel::right("test_script_right_".to_string() + id.as_str())
                .resizable(true)
                .min_width(300.0)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.label("Test scripts are written in JavaScript, and are run after the response is received.");
                    ui.separator();
                    ui.strong("SNIPPETS");
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            if ui.link("Log info message").clicked() {
                                script = script.clone().add("\nconsole.log(\"info1\",\"info2\");");
                            }
                            if ui.link("Log warn message").clicked() {
                                script = script.clone().add("\nconsole.warn(\"info1\",\"info2\");");
                            }
                            if ui.link("Log error message").clicked() {
                                script = script.clone().add("\nconsole.error(\"error1\",\"error2\");");
                            }
                            if ui.link("Get a variable").clicked() {
                                script = script.clone().add("\npostcat.get_env(\"variable_key\");");
                            }
                            if ui.link("Set a variable").clicked() {
                                script = script.clone().add("\npostcat.set_env(\"variable_key\",\"variable_value\");");
                            }
                            if ui.link("Get a shared").clicked() {
                                script = script.clone().add("\npostcat.get_shared(\"shared_key\");");
                            }
                            if ui.link("Set a shared").clicked() {
                                script = script.clone().add("\npostcat.set_shared(\"shared_key\",\"shared_value\");");
                            }
                            if ui.link("Get response").clicked() {
                                script = script.clone().add("\nlet response = postcat.resp();\nconsole.log(response)");
                            }
                        });
                });
            egui::SidePanel::left("test_script_left_".to_string() + id.as_str())
                .resizable(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| {
                    ui.push_id("test_script", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut script)
                                        .font(egui::TextStyle::Monospace) // for cursor height
                                        .code_editor()
                                        .desired_rows(10)
                                        .lock_focus(true)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter),
                                );
                            });
                    });
                });
        });
        script
    }
}
