use std::collections::{BTreeMap, HashMap};

use egui::{Color32, RichText, Ui};
use serde_json::Value;

use egui_code_editor::{CodeEditor, ColorTheme};
use netpurr_core::data::collections::Testcase;
use netpurr_core::data::workspace_data::WorkspaceData;

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::test_script_panel::CrtOrFolder;
use crate::utils;
use crate::widgets::syntax::js_syntax;

#[derive(Default)]
pub struct ManagerTestcaseWindows {
    open: bool,
    crt_or_folder: Option<CrtOrFolder>,
    select: Option<String>,
    new_case_name: String,
    source: String,
    message: RichText,
}
impl ManagerTestcaseWindows {
    pub fn with_crt_or_folder(mut self, crt_or_folder: CrtOrFolder) -> Self {
        self.crt_or_folder = Some(crt_or_folder);
        self
    }
}
impl Window for ManagerTestcaseWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("MANAGER TESTCASE")
            .modal(true)
            .min_width(800.0)
            .max_width(800.0)
            .max_height(400.0)
    }

    fn set_open(&mut self, open: bool) {
        self.open = open
    }

    fn get_open(&self) -> bool {
        self.open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        let mut testcases = BTreeMap::new();
        if let Some(crt_or_folder) = &self.crt_or_folder {
            match crt_or_folder {
                CrtOrFolder::CRT(crt_id) => {
                    testcases = workspace_data
                        .must_get_crt(crt_id.clone())
                        .record
                        .testcase()
                }
                CrtOrFolder::Folder(folder) => {
                    testcases = folder.borrow().testcases.clone();
                }
            }
            egui::panel::SidePanel::left("manager_testcase_left")
                .max_width(200.0)
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
                                self.source =
                                    serde_json::to_string_pretty(&testcase.value).unwrap();
                            };
                            select_value.context_menu(|ui| {
                                if ui.button("Remove").clicked() {
                                    remove_name_op = Some(name);
                                    ui.close_menu();
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
                    .max_height(400.0)
                    .show(ui, |ui| {
                        if code_editor.show(ui, &mut self.source).response.changed() {
                            match serde_json::from_str::<HashMap<String, Value>>(&self.source) {
                                Ok(testcase_value) => {
                                    self.message = RichText::new("Auto save success.")
                                        .color(Color32::DARK_GREEN);
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
            match crt_or_folder {
                CrtOrFolder::CRT(crt_id) => {
                    workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                        crt.record.set_testcases(testcases.clone());
                    });
                }
                CrtOrFolder::Folder(folder) => {
                    folder.borrow_mut().testcases = testcases.clone();
                }
            };
        }
    }
}
