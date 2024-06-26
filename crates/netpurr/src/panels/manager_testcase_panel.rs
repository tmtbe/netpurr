use std::collections::{BTreeMap, HashMap};

use eframe::epaint::Color32;
use egui::{RichText, Ui};
use serde_json::Value;

use egui_code_editor::{CodeEditor, ColorTheme};
use netpurr_core::data::collections::Testcase;
use netpurr_core::data::workspace_data::{TestItem, WorkspaceData};

use crate::utils;
use crate::widgets::syntax::js_syntax;

#[derive(Default)]
pub struct ManagerTestcasePanel {
    open: bool,
    select: Option<String>,
    new_case_name: String,
    source: String,
    message: RichText,
    test_item: Option<TestItem>,
    old_test_item_name: String,
    need_edit_name:Option<String>,
    edit_name:String,
    need_focus_edit:bool,
}
impl ManagerTestcasePanel {
    pub fn clear(&mut self) {
        self.source = "".to_string();
        self.select = None;
        self.test_item = None;
        self.need_edit_name = None;
    }
    pub fn render(&mut self, ui: &mut Ui, workspace_data: &mut WorkspaceData, test_item: TestItem) {
        let mut testcases = BTreeMap::new();
        let mut is_change = false;
        let mut test_item_name = "".to_string();
        match &test_item {
            TestItem::Record(_, folder, record_name) => {
                testcases = folder.borrow().requests[record_name].testcase();
                test_item_name = format!("{}/{}", folder.borrow().get_path(), record_name);
            }
            TestItem::Folder(_, folder) => {
                testcases = folder.borrow().testcases.clone();
                test_item_name = folder.borrow().get_path();
            }
        }
        if self.old_test_item_name != test_item_name {
            self.clear();
            self.old_test_item_name = test_item_name;
        }
        egui::panel::SidePanel::left("manager_testcase_left")
            .max_width(150.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_case_name);
                        ui.add_enabled_ui(
                            !testcases.contains_key(self.new_case_name.as_str()),
                            |ui| {
                                if ui.button("+").clicked() {
                                    if !self.new_case_name.is_empty() {
                                        testcases.insert(
                                            self.new_case_name.clone(),
                                            Testcase {
                                                entry_name: "".to_string(),
                                                name: self.new_case_name.clone(),
                                                value: Default::default(),
                                                parent_path: vec![],
                                            },
                                        );
                                        is_change = true;
                                    }
                                    self.new_case_name.clear();
                                }
                            },
                        );
                    });
                    let mut remove_name_op = None;
                    let testcases_clone = testcases.clone();
                    egui::scroll_area::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui,|ui|{
                        for (name, testcase) in testcases_clone.iter() {
                            if self.need_edit_name == Some(name.to_string()){
                                let editor = ui.text_edit_singleline(&mut self.edit_name);
                                if self.need_focus_edit{
                                    editor.request_focus();
                                    self.need_focus_edit = false;
                                }
                                if editor.lost_focus(){
                                    if !self.edit_name.is_empty()&&!testcases.contains_key(self.edit_name.as_str()){
                                        let mut new_testcase = testcase.clone();
                                        new_testcase.name = self.edit_name.clone();
                                        testcases.insert(self.edit_name.clone(),new_testcase);
                                        self.need_edit_name = None;
                                        remove_name_op = Some(name);
                                        is_change = true;
                                    }
                                }
                            }else {
                                let select_value = utils::select_value(
                                    ui,
                                    &mut self.select,
                                    Some(name.clone()),
                                    name.clone(),
                                );
                                if select_value.clicked() {
                                    self.source = serde_json::to_string_pretty(&testcase.value).unwrap();
                                };
                                select_value.context_menu(|ui| {
                                    if ui.button("Rename").clicked() {
                                        self.need_edit_name = Some(name.to_string());
                                        self.edit_name = name.to_string();
                                        self.need_focus_edit = true;
                                        ui.close_menu();
                                    }
                                    if ui.button("Remove").clicked() {
                                        remove_name_op = Some(name);
                                        ui.close_menu();
                                        is_change = true;
                                    }
                                });
                            }
                        }
                    });

                    if let Some(remove_name) = remove_name_op {
                        testcases.remove(remove_name);
                    };
                })
            });
        ui.label(self.message.clone());
        let mut code_editor = CodeEditor::default()
            .id_source(format!("{}:{}","testcase_code_editor",self.select.clone().unwrap_or_default()))
            .with_rows(25)
            .with_ui_fontsize(ui)
            .with_syntax(js_syntax())
            .with_numlines(true);
        if ui.visuals().dark_mode {
            code_editor = code_editor.with_theme(ColorTheme::GRUVBOX)
        } else {
            code_editor = code_editor.with_theme(ColorTheme::GRUVBOX_LIGHT)
        }
        if let Some(select) = &self.select {
            egui::ScrollArea::vertical()
                .min_scrolled_height(200.0)
                .max_height(400.0)
                .show(ui, |ui| {
                    let text_edit_output= code_editor.show(ui, &mut self.source);
                    if text_edit_output.response.lost_focus(){
                        match serde_json::from_str::<HashMap<String, Value>>(&self.source) {
                            Ok(testcase_value) => {
                                self.source =
                                    serde_json::to_string_pretty(&testcase_value).unwrap();
                            }
                            Err(_)=>{

                            }
                        }
                    }
                    if text_edit_output.response.changed() {
                        match serde_json::from_str::<HashMap<String, Value>>(&self.source) {
                            Ok(testcase_value) => {
                                self.message =
                                    RichText::new("Auto save success.").color(Color32::DARK_GREEN);
                                is_change = true;
                                testcases.insert(
                                    select.to_string(),
                                    Testcase {
                                        entry_name: "".to_string(),
                                        name: select.to_string(),
                                        value: testcase_value,
                                        parent_path: vec![],
                                    },
                                );
                            }
                            Err(e) => {
                                self.message =
                                    RichText::new(e.to_string()).color(Color32::DARK_RED);
                            }
                        }
                    }
                });
        } else {
            let mut text =
                "Select one testcase to edit, the testcase format is `json`.".to_string();
            code_editor.show(ui, &mut text);
        };
        match &test_item {
            TestItem::Record(_, folder, record_name) => {
                folder
                    .borrow_mut()
                    .requests
                    .get_mut(record_name)
                    .unwrap()
                    .set_testcases(testcases.clone());
                if is_change {
                    workspace_data.save_record(folder.clone(), record_name.clone());
                }
            }
            TestItem::Folder(_, folder) => {
                folder.borrow_mut().testcases = testcases.clone();
                if is_change {
                    workspace_data.save_folder(folder.clone())
                }
            }
        };
    }
}
