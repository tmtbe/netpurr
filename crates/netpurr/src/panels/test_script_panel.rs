use std::cell::RefCell;
use std::ops::Add;
use std::rc::Rc;

use crate::data::workspace::Workspace;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use egui::Ui;
use egui_code_editor::{CodeEditor, ColorTheme};
use netpurr_core::data::collections::{CollectionFolder, Testcase};

use crate::widgets::syntax::js_syntax;
use crate::windows::manager_testcase_window::ManagerTestcaseWindows;

#[derive(Default)]
pub struct TestScriptPanel {}

#[derive(Clone)]
pub enum CrtOrFolder {
    CRT(String),
    Folder(Rc<RefCell<CollectionFolder>>),
}
impl TestScriptPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        crt_or_folder: CrtOrFolder,
        id: String,
    ) {
        let mut script = "".to_string();
        match &crt_or_folder {
            CrtOrFolder::CRT(crt_id) => {
                script = workspace_data
                    .must_get_crt(crt_id.clone())
                    .record
                    .test_script();
            }
            CrtOrFolder::Folder(folder) => {
                script = folder.borrow().test_script.clone();
            }
        }
        ui.vertical(|ui| {
            ui.label("Test scripts are written in JavaScript, and are run after the response is received.");
            ui.separator();
            ui.horizontal(|ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(250.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if ui.button("Manager Testcases").clicked(){
                                operation.add_window(Box::new(ManagerTestcaseWindows::default().with_crt_or_folder(crt_or_folder.clone())))
                            }
                            ui.strong("SNIPPETS");
                            if ui.link("Test Example").clicked() {
                                script = script.clone().add(r#"let response = netpurr.resp();
console.log(response);
netpurr.test("This is a test example",function(){
// assert("expect", "actual");
   assert("test",response.json.cookies.freeform);
});
                        "#)
                            }
                            if ui.link("Status is 200").clicked(){
                                script = script.clone().add(r#"
let response = netpurr.resp();
netpurr.test("Response status is 200",function(){
    assert(200,response.status);
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
                            if ui.link("Wait a shared").clicked() {
                                script = script.clone().add("\nawait netpurr.wait_shared(\"shared_key\");");
                            }
                            if ui.link("Set a shared").clicked() {
                                script = script.clone().add("\nnetpurr.set_shared(\"shared_key\",\"shared_value\");");
                            }
                            if ui.link("Sleep").clicked() {
                                script = script.clone().add("\nawait sleep(1000);");
                            }
                            if ui.link("Get response").clicked() {
                                script = script.clone().add("\nlet response = netpurr.resp();\nconsole.log(response)");
                            }
                        });
                    });
                ui.separator();
                ui.vertical(|ui|{
                    ui.push_id(id+"_test_script", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(250.0)
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
        });
        match &crt_or_folder {
            CrtOrFolder::CRT(crt_id) => {
                workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                    crt.record.set_test_script(script);
                });
            }
            CrtOrFolder::Folder(folder) => folder.borrow_mut().test_script = script,
        }
    }
}
