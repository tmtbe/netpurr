use std::ops::Add;

use egui::Ui;
use egui_code_editor::{CodeEditor, ColorTheme};

use crate::panels::HORIZONTAL_GAP;
use crate::widgets::syntax::js_syntax;

#[derive(Default)]
pub struct TestScriptPanel {}

impl TestScriptPanel {
    pub fn set_and_render(&mut self, ui: &mut Ui, mut script: String, id: String) -> String {
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
                            if ui.link("Test Example").clicked() {
                                script = script.clone().add(r#"let response = netpurr.resp();                                
console.log(response);
netpurr.test("This is a test example",function(){
    // assert("expect", "actual");
	assert("test",response.json.cookies.freeform);
});
                                "#)
                            }
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
                                script = script.clone().add("\nnetpurr.get_env(\"variable_key\");");
                            }
                            if ui.link("Set a variable").clicked() {
                                script = script.clone().add("\nnetpurr.set_env(\"variable_key\",\"variable_value\");");
                            }
                            if ui.link("Get a shared").clicked() {
                                script = script.clone().add("\nnetpurr.get_shared(\"shared_key\");");
                            }
                            if ui.link("Set a shared").clicked() {
                                script = script.clone().add("\nnetpurr.set_shared(\"shared_key\",\"shared_value\");");
                            }
                            if ui.link("Get response").clicked() {
                                script = script.clone().add("\nlet response = netpurr.resp();\nconsole.log(response)");
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
                                let mut code_editor = CodeEditor::default()
                                    .id_source("test_code_editor")
                                    .with_rows(12)
                                    .with_ui_fontsize(ui)
                                    .with_syntax(js_syntax())
                                    .with_numlines(true);
                                if ui.visuals().dark_mode {
                                    code_editor = code_editor.with_theme(ColorTheme::GRUVBOX)
                                } else {
                                    code_editor = code_editor.with_theme(ColorTheme::GRUVBOX_LIGHT)
                                }
                                code_editor.show(ui, &mut script);
                            });
                    });
                });
        });
        script
    }
}
