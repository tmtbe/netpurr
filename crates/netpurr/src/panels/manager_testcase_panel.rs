use crate::utils;
use crate::widgets::syntax::js_syntax;
use eframe::epaint::Color32;
use egui::{RichText, Ui};
use egui_code_editor::{CodeEditor, ColorTheme};
use netpurr_core::data::collections::Testcase;
use netpurr_core::data::workspace_data::{TestItem, WorkspaceData};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fmt::format;

#[derive(Default)]
pub struct ManagerTestcasePanel {
    open: bool,
    select: Option<String>,
    new_case_name: String,
    source: String,
    message: RichText,
    test_item: Option<TestItem>,
    old_test_item_name: String,
}
impl ManagerTestcasePanel {
    pub fn clear(&mut self) {
        self.source = "".to_string();
        self.select = None;
        self.test_item = None;
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
                    for (name, testcase) in testcases_clone.iter() {
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
                            if ui.button("Remove").clicked() {
                                remove_name_op = Some(name);
                                ui.close_menu();
                                is_change = true;
                            }
                        });
                    }
                    if let Some(remove_name) = remove_name_op {
                        testcases.remove(remove_name);
                    };
                })
            });
        ui.label(self.message.clone());
        let mut code_editor = CodeEditor::default()
            .id_source("testcase_code_editor")
            .with_rows(12)
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
                    if code_editor.show(ui, &mut self.source).response.changed() {
                        match serde_json::from_str::<HashMap<String, Value>>(&self.source) {
                            Ok(testcase_value) => {
                                self.message =
                                    RichText::new("Auto save success.").color(Color32::DARK_GREEN);
                                is_change = true;
                                self.source =
                                    serde_json::to_string_pretty(&testcase_value).unwrap();
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
