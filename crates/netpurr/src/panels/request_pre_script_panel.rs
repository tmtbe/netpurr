use std::ops::Add;

use egui::Ui;
use prettify_js::prettyprint;

use egui_code_editor::{CodeEditor, ColorTheme, Prompt};
use netpurr_core::data::workspace_data::{TestItem, WorkspaceData};

use crate::widgets::syntax::js_syntax;

#[derive(Default)]
pub struct RequestPreScriptPanel {}

impl RequestPreScriptPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        test_item: TestItem,
    ) {
        let mut script = "".to_string();
        match &test_item {
            TestItem::Folder(_, folder) => {
                script = folder.borrow().pre_request_script.clone();
            }
            TestItem::Record(_, folder, record_name) => {
                folder
                    .borrow()
                    .requests
                    .get(record_name.as_str())
                    .map(|record| script = record.pre_request_script());
            }
        }
        let compare_script = script.clone();
        egui::panel::SidePanel::left("manager_testcase_left")
            .max_width(150.0)
            .show_inside(ui, |ui| {
                ui.label("Pre-request scripts are written in JavaScript， and are run before the request is sent.");
                ui.separator();
                ui.horizontal(|ui| {
                    egui::ScrollArea::vertical().min_scrolled_height(250.0)
                        .id_source("test_manager_pre_request_script_snippets")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.strong("SNIPPETS");
                                if ui.link("Get testcase info").clicked() {
                                    script = script.clone().add("\nlet value = netpurr.get_testcase().key;");
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
                                if ui.link("Add a header").clicked() {
                                    script = script.clone().add("\nnetpurr.add_header(\"header_key\",\"header_value\");");
                                }
                                if ui.link("Add a params").clicked() {
                                    script = script.clone().add("\nnetpurr.add_params(\"params_key\",\"params_value\");");
                                }
                                if ui.link("Get a shared").clicked() {
                                    script = script.clone().add("\nnetpurr.get_shared(\"shared_key\");");
                                }
                                if ui.link("Wait a shared").clicked() {
                                    script = script.clone().add("\nawait netpurr.wait_shared(\"shared_key\");");
                                }
                                if ui.link("Set a shared").clicked() {
                                    script = script.clone().add("\nnetpurr.set_shared(\"shared_key\",\"shared_value\");");
                                }
                                if ui.link("Sleep").clicked() {
                                    script = script.clone().add("\nawait sleep(1000);");
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
            });
        });
        ui.push_id("pre_request_script", |ui| {
            egui::ScrollArea::vertical()
                .min_scrolled_height(ui.available_height()-30.0)
                .id_source("test_manager_pre_request_script")
                .show(ui, |ui| {
                    let prompt_yaml = include_str!("../../prompt/js.yaml");
                    let mut code_editor = CodeEditor::default()
                        .id_source("request_pre_script_code_editor")
                        .with_rows(25)
                        .with_ui_fontsize(ui)
                        .with_syntax(js_syntax())
                        .with_prompt(Prompt::from_str(prompt_yaml))
                        .with_numlines(true);
                    if ui.visuals().dark_mode {
                        code_editor = code_editor.with_theme(ColorTheme::GRUVBOX)
                    } else {
                        code_editor = code_editor.with_theme(ColorTheme::GRUVBOX_LIGHT)
                    }
                    let response = code_editor.show(ui, &mut script).response;
                    if response.lost_focus(){
                        let (pretty, _) = prettyprint(script.as_str());
                        script = pretty;
                    }
                });
        });
        if compare_script != script {
            match &test_item {
                TestItem::Folder(_, folder) => {
                    folder.borrow_mut().pre_request_script = script.clone();
                    workspace_data.save_folder(folder.clone());
                }
                TestItem::Record(_, folder, record_name) => {
                    folder
                        .borrow_mut()
                        .requests
                        .get_mut(record_name.as_str())
                        .map(|record| {
                            record.set_pre_request_script(script.clone());
                        });
                    workspace_data.save_record(folder.clone(), record_name.clone());
                }
            }
        }
    }
}
