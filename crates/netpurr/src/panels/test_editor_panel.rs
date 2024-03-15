use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use eframe::epaint::{Color32, FontFamily, FontId};
use egui::text::LayoutJob;
use egui::{Align, FontSelection, RichText, Style, Ui};
use poll_promise::Promise;

use netpurr_core::data::collections::{CollectionFolder, Testcase};
use netpurr_core::data::test::TestStatus;
use netpurr_core::runner::test::{ResultTreeCase, ResultTreeFolder, ResultTreeRequest};
use netpurr_core::runner::TestGroupRunResults;

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::utils;

#[derive(Default)]
pub struct TestEditorPanel {
    test_group_run_result: Option<Arc<RwLock<TestGroupRunResults>>>,
    run_promise: Option<Promise<()>>,
    collection_path: String,
    parent_testcase_list: Vec<Testcase>,
    parent_paths: Vec<String>,
}
impl TestEditorPanel {
    pub fn render(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
    ) {
        if let Some(p) = &self.run_promise {
            if p.ready().is_some() {
                self.run_promise = None
            }
        }
        if self.run_promise.is_some() {
            ui.ctx().request_repaint();
        }
        workspace_data
            .selected_test_group_path
            .clone()
            .map(|collection_path| {
                if let (collection_name, Some(folder)) =
                    workspace_data.get_folder_with_path(collection_path.clone())
                {
                    if self.collection_path != collection_path {
                        self.init(workspace_data, collection_path.clone());
                    }
                    ui.heading(folder.borrow().name.clone());
                    ui.separator();
                    ui.add_enabled_ui(self.run_promise.is_none(), |ui| {
                        self.render_select_testcase(workspace_data, ui);
                        if ui.button("Run").clicked() {
                            let test_group_run_result =
                                Arc::new(RwLock::new(TestGroupRunResults::default()));
                            self.test_group_run_result = Some(test_group_run_result.clone());
                            self.run_test_group(
                                workspace_data,
                                operation,
                                test_group_run_result,
                                collection_name,
                                collection_path,
                                self.build_parent_testcase(),
                                folder.clone(),
                            );
                        }
                    });
                    if let Some(test_group_run_result) = self.test_group_run_result.clone() {
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 30.0)
                            .show(ui, |ui| {
                                let mut testcase_paths = vec![];
                                if let Some(pt) = self.build_parent_testcase() {
                                    testcase_paths = pt.get_testcase_path();
                                }
                                let result_tree = ResultTreeFolder::create(
                                    folder.clone(),
                                    testcase_paths,
                                    test_group_run_result.read().unwrap().deref().clone(),
                                );
                                self.render_tree_folder(ui, workspace_data, &result_tree);
                            });
                    }
                }
            });
    }

    fn render_select_testcase(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for (index, parent_path) in self.parent_paths.iter().enumerate() {
                let (_, folder_op) = workspace_data.get_folder_with_path(parent_path.clone());
                if let Some(folder) = folder_op {
                    let mut testcases = folder.borrow().testcases.clone();
                    if testcases.is_empty() {
                        let testcase = Testcase::default();
                        testcases.insert(testcase.name.clone(), testcase);
                    }
                    egui::ComboBox::from_id_source(format!("testcase-{}", parent_path))
                        .selected_text(self.parent_testcase_list[index].clone().get_path())
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            for (name, testcase) in testcases.iter() {
                                let testcase_path = format!("{}:{}", folder.borrow().name, name);
                                let mut response = ui.selectable_label(
                                    self.parent_testcase_list[index].get_path().as_str()
                                        == testcase_path.as_str(),
                                    testcase_path.clone(),
                                );
                                if response.clicked() {
                                    let mut new_testcase = testcase.clone();
                                    new_testcase.entry_name = folder.borrow().name.clone();
                                    self.parent_testcase_list[index] = new_testcase;
                                    response.mark_changed();
                                }
                            }
                        });
                }
            }
        });
    }

    fn init(&mut self, workspace_data: &mut WorkspaceData, collection_path: String) {
        self.collection_path = collection_path.clone();
        self.parent_testcase_list.clear();
        self.parent_paths.clear();
        let collection_path_split: Vec<String> =
            collection_path.split("/").map(|s| s.to_string()).collect();
        for index in 0..collection_path_split.len() {
            if index == collection_path_split.len() || index == 0 {
                continue;
            }
            let path: Vec<String> = collection_path_split.iter().take(index).cloned().collect();
            let path_join = path.join("/");
            self.parent_paths.push(path_join.clone());

            let (_, folder_op) = workspace_data.get_folder_with_path(path_join.clone());
            if let Some(folder) = folder_op {
                let mut testcases = folder.borrow().testcases.clone();
                if testcases.is_empty() {
                    let testcase = Testcase::default();
                    testcases.insert(testcase.name.clone(), testcase);
                }
                let mut select_testcase = testcases.first_entry().unwrap().get().clone();
                select_testcase.entry_name = folder.borrow().name.clone();
                self.parent_testcase_list.push(select_testcase)
            }
        }
    }

    fn build_parent_testcase(&self) -> Option<Testcase> {
        let mut merge_testcase: Option<Testcase> = None;
        for testcase in self.parent_testcase_list.iter() {
            match &merge_testcase {
                Some(t) => {
                    let mut next_testcase = testcase.clone();
                    next_testcase.merge(testcase.entry_name.clone(), t);
                    merge_testcase = Some(next_testcase);
                }
                None => {
                    merge_testcase = Some(testcase.clone());
                }
            }
        }
        merge_testcase
    }

    fn render_tree_case(
        &self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        result_tree_case: &ResultTreeCase,
    ) {
        let title = self.render_test_title(
            ui,
            format!(
                "{} ({}/{})",
                result_tree_case.name.clone(),
                result_tree_case.get_success_count(),
                result_tree_case.get_total_count()
            ),
            result_tree_case.status.clone(),
        );
        utils::open_collapsing(ui, title, |child_ui| {
            for (name, rf) in result_tree_case.folders.iter() {
                self.render_tree_folder(child_ui, workspace_data, rf)
            }
            for request_tree_request in result_tree_case.requests.iter() {
                self.render_test_request(child_ui, workspace_data, request_tree_request);
            }
        });
    }
    fn render_tree_folder(
        &self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        result_tree_folder: &ResultTreeFolder,
    ) {
        let title = self.render_test_title(
            ui,
            format!(
                "{} ({}/{})",
                result_tree_folder.name.clone(),
                result_tree_folder.get_success_count(),
                result_tree_folder.get_total_count()
            ),
            result_tree_folder.status.clone(),
        );
        utils::open_collapsing(ui, title, |child_ui| {
            for (_, case) in result_tree_folder.cases.iter() {
                self.render_tree_case(child_ui, workspace_data, case);
            }
        });
    }

    fn render_test_request(
        &self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        request_tree_request: &ResultTreeRequest,
    ) {
        let request_title = self.render_test_title(
            ui,
            request_tree_request.name.clone(),
            request_tree_request.status.clone(),
        );
        if ui
            .collapsing(request_title, |ui| {
                // 增加测试的信息
                if let Some(r) = &request_tree_request.result {
                    match r {
                        Ok(tr) => {
                            for test_info in tr.test_result.test_info_list.iter() {
                                let test_info_title = self.render_test_title(
                                    ui,
                                    test_info.name.clone(),
                                    test_info.status.clone(),
                                );
                                ui.collapsing(test_info_title, |ui| {
                                    for tar in test_info.results.iter() {
                                        let test_assert_title = self.render_test_title(
                                            ui,
                                            tar.msg.clone(),
                                            tar.assert_result.clone(),
                                        );
                                        ui.label(test_assert_title);
                                    }
                                });
                            }
                        }
                        Err(te) => {
                            let test_info_title = self.render_test_title(
                                ui,
                                te.error.clone(),
                                TestStatus::FAIL.clone(),
                            );
                            ui.label(test_info_title);
                        }
                    }
                }
            })
            .header_response
            .clicked()
        {
            if let Some(r) = &request_tree_request.result {
                if let Ok(tr) = r {
                    workspace_data.selected_test_run_result = Some(tr.clone())
                }
            }
        };
    }

    fn render_test_title(&self, ui: &mut Ui, name: String, status: TestStatus) -> LayoutJob {
        let style = Style::default();
        let mut request_test_result_name_layout_job = LayoutJob::default();
        let mut rich_text = RichText::new(status.clone().to_string())
            .color(Color32::WHITE)
            .font(FontId {
                size: 14.0,
                family: FontFamily::Monospace,
            });
        match status {
            TestStatus::None => {
                rich_text = rich_text.background_color(ui.visuals().extreme_bg_color)
            }
            TestStatus::PASS => rich_text = rich_text.background_color(Color32::DARK_GREEN),
            TestStatus::FAIL => rich_text = rich_text.background_color(Color32::DARK_RED),
            TestStatus::Waiting => rich_text = rich_text.background_color(Color32::DARK_BLUE),
        };
        rich_text.append_to(
            &mut request_test_result_name_layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
        RichText::new("  ".to_string() + name.as_str()).append_to(
            &mut request_test_result_name_layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
        request_test_result_name_layout_job
    }

    fn run_test_group(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_name: String,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        let envs = workspace_data
            .get_build_envs(workspace_data.get_collection(Some(collection_name.clone())));
        let (pre_request_parent_script_scopes, test_parent_script_scopes) =
            workspace_data.get_parent_scripts(collection_path.clone());
        self.run_promise = Some(operation.run_test_group_promise(
            envs,
            pre_request_parent_script_scopes,
            test_parent_script_scopes,
            test_group_run_result,
            collection_name,
            collection_path,
            parent_testcase,
            folder,
        ));
    }
}
