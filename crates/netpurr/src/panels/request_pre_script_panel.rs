use std::collections::BTreeMap;
use std::ops::Add;

use egui::Ui;
use egui_code_editor::{CodeEditor, ColorTheme};

use netpurr_core::data::environment::EnvironmentItemValue;
use netpurr_core::data::http::Request;
use netpurr_core::script::{Context, ScriptScope};

use crate::operation::operation::Operation;
use crate::panels::HORIZONTAL_GAP;
use crate::widgets::syntax::js_syntax;
use crate::windows::test_script_windows::TestScriptWindows;

#[derive(Default)]
pub struct RequestPreScriptPanel {}

impl RequestPreScriptPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        scope: String,
        mut script: String,
        mut parent_scripts: Vec<ScriptScope>,
        request: Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        id: String,
    ) -> String {
        ui.horizontal(|ui| {
            egui::SidePanel::right("pre_request_right_".to_string() + id.as_str())
                .resizable(true)
                .min_width(300.0)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.label("Pre-request scripts are written in JavaScript， and are run before the request is sent.");
                    if ui.link("Test").clicked() {
                        let script_scope = ScriptScope {
                            script: script.clone(),
                            scope: scope.clone(),
                        };
                        parent_scripts.push(script_scope);
                        let context = Context {
                            scope_name: scope.clone(),
                            request: request.clone(),
                            envs,
                            ..Default::default()
                        };
                        operation.add_window(Box::new(TestScriptWindows::default().with(parent_scripts, context)));
                    }
                    ui.separator();
                    ui.strong("SNIPPETS");
                    egui::ScrollArea::vertical().show(ui, |ui| {
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
                        if ui.link("Add a header").clicked() {
                            script = script.clone().add("\nnetpurr.add_header(\"header_key\",\"header_value\");");
                        }
                        if ui.link("Add a params").clicked() {
                            script = script.clone().add("\nnetpurr.add_params(\"params_key\",\"params_value\");");
                        }
                        if ui.link("Get a shared").clicked() {
                            script = script.clone().add("\nnetpurr.get_shared(\"shared_key\");");
                        }
                        if ui.link("Set a shared").clicked() {
                            script = script.clone().add("\nnetpurr.set_shared(\"shared_key\",\"shared_value\");");
                        }
                        if ui.link("Fetch a http request").clicked() {
                            script = script.clone().add(
                                r#"let request = {
    "method":"post",
    "url":"http://www.httpbin.org/post",
    "headers":[{
        "name":"name",
        "value":"value"
    }],
    "body":"body"
}
let response = await fetch(request);
console.log(response)"#)
                        }
                    });
                });
            egui::SidePanel::left("pre_request_left_".to_string() + id.as_str())
                .resizable(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| {
                    ui.push_id("pre_request_script", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .show(ui, |ui| {
                                let mut code_editor = CodeEditor::default()
                                    .id_source("request_pre_script_code_editor")
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
