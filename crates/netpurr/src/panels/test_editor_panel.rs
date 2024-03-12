use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use eframe::epaint::{Color32, FontFamily, FontId};
use egui::text::LayoutJob;
use egui::{Align, FontSelection, RichText, Style, Ui};
use poll_promise::Promise;

use netpurr_core::data::collections::{CollectionFolder, Testcase};
use netpurr_core::data::test::TestStatus;
use netpurr_core::runner::{TestGroupRunResults, TestRunError, TestRunResult};

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::utils;

#[derive(Default)]
pub struct TestEditorPanel {
    test_group_run_result: Option<Arc<RwLock<TestGroupRunResults>>>,
    run_promise: Option<Promise<()>>,
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
                    ui.heading(folder.borrow().name.clone());
                    ui.separator();
                    ui.add_enabled_ui(self.run_promise.is_none(), |ui| {
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
                                folder.clone(),
                            );
                        }
                    });
                    if let Some(test_group_run_result) = self.test_group_run_result.clone() {
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 30.0)
                            .show(ui, |ui| {
                                let result_tree = ResultTreeFolder::create(
                                    folder.clone(),
                                    test_group_run_result.read().unwrap().deref().clone(),
                                );
                                self.render_tree_folder(ui, workspace_data, &result_tree);
                            });
                    }
                }
            });
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
            folder,
        ));
    }
}

#[derive(Default, Clone)]
struct ResultTreeFolder {
    status: TestStatus,
    name: String,
    cases: BTreeMap<String, ResultTreeCase>,
}
#[derive(Default, Clone)]
struct ResultTreeCase {
    status: TestStatus,
    name: String,
    folders: BTreeMap<String, ResultTreeFolder>,
    requests: Vec<ResultTreeRequest>,
}
impl ResultTreeCase {
    fn get_success_count(&self) -> i32 {
        let mut success_count = 0;
        for r in self.requests.iter() {
            if r.status == TestStatus::PASS {
                success_count = success_count + 1;
            }
        }
        for (_, f) in self.folders.iter() {
            success_count += f.get_success_count();
        }
        success_count
    }
    fn get_total_count(&self) -> i32 {
        let mut success_count = 0;

        for r in self.requests.iter() {
            success_count = success_count + 1;
        }
        for (_, f) in self.folders.iter() {
            success_count += f.get_total_count();
        }

        success_count
    }
}
impl ResultTreeFolder {
    pub fn create(folder: Rc<RefCell<CollectionFolder>>, results: TestGroupRunResults) -> Self {
        let mut folder_status = TestStatus::Waiting;
        let mut testcases = folder.borrow().testcases.clone();
        if testcases.is_empty() {
            let testcase = Testcase::default();
            testcases.insert(testcase.name.clone(), testcase);
        }
        let mut new_result_tree_folder = ResultTreeFolder {
            status: folder_status.clone(),
            name: folder.borrow().name.clone(),
            cases: Default::default(),
        };
        folder_status = TestStatus::PASS;
        for (name, testcase) in testcases.iter() {
            let mut case_status = TestStatus::Waiting;
            let mut case_folders = BTreeMap::new();
            let mut case_requests = vec![];
            case_status = TestStatus::PASS;
            for (name, f) in folder.borrow().folders.iter() {
                let child_folder = ResultTreeFolder::create(f.clone(), results.clone());
                match &child_folder.status {
                    TestStatus::None => {}
                    TestStatus::Waiting => case_status = TestStatus::Waiting,
                    TestStatus::PASS => {}
                    TestStatus::FAIL => case_status = TestStatus::FAIL,
                }
                case_folders.insert(name.to_string(), child_folder);
            }
            for (name, _) in folder.borrow().requests.iter() {
                let result = results.find(
                    folder.borrow().get_path(),
                    testcase.name.clone(),
                    name.clone(),
                );
                let mut request_status = TestStatus::Waiting;
                match &result {
                    None => {
                        case_status = TestStatus::Waiting;
                    }
                    Some(rr) => match rr {
                        Ok(r) => {
                            request_status = r.test_result.status.clone();
                            if request_status == TestStatus::FAIL {
                                case_status = TestStatus::FAIL;
                            }
                        }
                        Err(e) => {
                            request_status = TestStatus::FAIL;
                            case_status = TestStatus::FAIL;
                        }
                    },
                }
                case_requests.push(ResultTreeRequest {
                    name: name.clone(),
                    status: request_status,
                    result: result.clone(),
                });
            }
            let result_tree_case = ResultTreeCase {
                status: case_status.clone(),
                name: testcase.name.to_string(),
                folders: case_folders.clone(),
                requests: case_requests.clone(),
            };
            new_result_tree_folder
                .cases
                .insert(testcase.name.to_string(), result_tree_case);
            match &case_status {
                TestStatus::None => {}
                TestStatus::Waiting => folder_status = TestStatus::Waiting,
                TestStatus::PASS => {}
                TestStatus::FAIL => folder_status = TestStatus::FAIL,
            }
        }
        new_result_tree_folder.status = folder_status.clone();
        new_result_tree_folder
    }

    fn get_success_count(&self) -> i32 {
        let mut success_count = 0;
        for (_, case) in self.cases.iter() {
            for r in case.requests.iter() {
                if r.status == TestStatus::PASS {
                    success_count = success_count + 1;
                }
            }
            for (_, f) in case.folders.iter() {
                success_count += f.get_success_count();
            }
        }
        success_count
    }
    fn get_total_count(&self) -> i32 {
        let mut success_count = 0;
        for (_, case) in self.cases.iter() {
            for r in case.requests.iter() {
                success_count = success_count + 1;
            }
            for (_, f) in case.folders.iter() {
                success_count += f.get_total_count();
            }
        }
        success_count
    }
}

#[derive(Default, Clone)]
struct ResultTreeRequest {
    name: String,
    status: TestStatus,
    result: Option<Result<TestRunResult, TestRunError>>,
}
