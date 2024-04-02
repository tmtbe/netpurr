use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use eframe::epaint::{Color32, FontFamily, FontId};
use egui::{Align, FontSelection, RichText, Style, Ui};
use egui::text::LayoutJob;
use poll_promise::Promise;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use netpurr_core::data::collections::{CollectionFolder, Testcase};
use netpurr_core::data::record::Record;
use netpurr_core::data::test::TestStatus;
use netpurr_core::data::workspace_data::{TestItem, WorkspaceData};
use netpurr_core::runner::{TestGroupRunResults, TestRunResult};
use netpurr_core::runner::test::{ResultTreeCase, ResultTreeFolder, ResultTreeRequest};

use crate::operation::operation::Operation;
use crate::panels::manager_testcase_panel::ManagerTestcasePanel;
use crate::panels::request_pre_script_panel::RequestPreScriptPanel;
use crate::panels::test_script_panel::TestScriptPanel;
use crate::utils;
use crate::utils::HighlightValue;

#[derive(Default)]
pub struct TestEditorPanel {
    manager_testcase_panel: ManagerTestcasePanel,
    request_pre_script_panel: RequestPreScriptPanel,
    test_script_panel: TestScriptPanel,
    test_group_run_result: Option<Arc<RwLock<TestGroupRunResults>>>,
    run_promise: Option<Promise<()>>,
    collection_path: String,
    parent_testcase_list: Vec<Testcase>,
    parent_paths: Vec<String>,
    open_panel_enum:Panel,
}
#[derive(Display)]
enum TitleType{
    Testcase,
    Request,
    Assert,
    Group
}

#[derive(EnumIter,Display,Clone,PartialEq)]
enum Panel{
    Runner,
    Testcase,
    PreRequestScript,
    TestScript,
}
impl Default for Panel{
    fn default() -> Self {
        Panel::Runner
    }
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
        workspace_data.selected_test_item.clone().map(|test_item| {
            self.render_test_item_folder(operation, workspace_data, ui, test_item.clone());
            ui.horizontal(|ui|{
                for panel in Panel::iter() {
                    let layout_job = self.build_panel_title(&panel,workspace_data,ui);
                    ui.selectable_value(
                        &mut self.open_panel_enum,
                        panel.clone(),
                        layout_job
                    );
                }
            });
            ui.separator();
            match self.open_panel_enum {
                Panel::Runner => {
                    match &test_item {
                        TestItem::Folder(collection_name, folder) => {
                            self.render_select_testcase(workspace_data, ui);
                            self.render_run_folder(
                                operation,
                                workspace_data,
                                ui,
                                collection_name.clone(),
                                &folder,
                            );
                            self.render_result_tree(workspace_data, ui, folder.clone());
                        }
                        TestItem::Record(collection_name, folder,record_name) => {
                            self.render_select_testcase(workspace_data, ui);
                            let record = folder.borrow().requests[record_name].clone();
                            self.render_run_record(
                                operation,
                                workspace_data,
                                ui,
                                collection_name.clone(),
                                folder.clone(),
                                record,
                            );
                            self.render_result_record(workspace_data, ui,folder,record_name);
                        }
                    }
                }
                Panel::Testcase => {
                    self.render_manager_testcase(workspace_data, ui, test_item.clone());
                }
                Panel::PreRequestScript => {
                    ui.strong("Pre-request Script:");
                    self.request_pre_script_panel
                        .set_and_render(ui, workspace_data, test_item.clone());
                }
                Panel::TestScript => {
                    ui.strong("Test Script:");
                    self.test_script_panel
                        .set_and_render(ui, workspace_data, test_item.clone())
                }
            }
        });
    }

    fn build_panel_title(&self,panel:&Panel,workspace_data: &mut WorkspaceData,ui: &mut Ui)->LayoutJob{
        return match &workspace_data.selected_test_item {
            None => {
                utils::build_with_count_ui_header(panel.to_string(), HighlightValue::None, ui)
            }
            Some(test_item) => {
                match panel {
                    Panel::Runner => {
                        utils::build_with_count_ui_header(panel.to_string(), HighlightValue::None, ui)
                    }
                    Panel::Testcase => {
                        match test_item {
                            TestItem::Folder(_, f) => {
                                utils::build_with_count_ui_header(panel.to_string(), HighlightValue::Usize(f.borrow().testcases.len()), ui)
                            }
                            TestItem::Record(_, f, r) => {
                                let size = f.borrow().requests.get(r).unwrap().testcase().len();
                                utils::build_with_count_ui_header(panel.to_string(), HighlightValue::Usize(size), ui)
                            }
                        }
                    }
                    Panel::PreRequestScript => {
                        match test_item {
                            TestItem::Folder(_, f) => {
                                if f.borrow().pre_request_script.is_empty() {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::None, ui)
                                } else {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::Has, ui)
                                }
                            }
                            TestItem::Record(_, f, r) => {
                                if f.borrow().requests.get(r).unwrap().pre_request_script().is_empty() {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::None, ui)
                                } else {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::Has, ui)
                                }
                            }
                        }
                    }
                    Panel::TestScript => {
                        match test_item {
                            TestItem::Folder(_, f) => {
                                if f.borrow().test_script.is_empty() {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::None, ui)
                                } else {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::Has, ui)
                                }
                            }
                            TestItem::Record(_, f, r) => {
                                if f.borrow().requests.get(r).unwrap().test_script().is_empty() {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::None, ui)
                                } else {
                                    utils::build_with_count_ui_header(panel.to_string(), HighlightValue::Has, ui)
                                }
                            }
                        }
                    }
                }
            }
        }

    }
    fn render_test_item_folder(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        test_item: TestItem,
    ) {
        match &test_item {
            TestItem::Folder(collection_name, folder) => {
                if self.collection_path != folder.borrow().get_path() {
                    self.init(workspace_data, folder.borrow().get_path());
                }
                ui.heading(format!("{}{}","FOLDER | ",folder.borrow().get_path().clone()));
                ui.separator();
            }
            TestItem::Record(collection_name, folder, record_name) => {
                let record_path = format!("{}/{}", folder.borrow().get_path(), record_name);
                if self.collection_path != record_path {
                    self.init(workspace_data, record_path);
                }
                ui.heading(format!("{}{} : {}","RECORD | ", folder.borrow().get_path(), record_name));
                ui.separator();
            }
        }
    }
    fn render_result_record(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui,folder:&Rc<RefCell<CollectionFolder>>,record_name: &String) {
        if let Some(test_group_run_result) = self.test_group_run_result.clone() {
            for (name, result) in test_group_run_result.read().unwrap().results.iter() {
                if !name.contains(record_name.as_str()){
                    continue;
                }
                let mut status = TestStatus::PASS;
                match result {
                    Ok(result) => status = result.test_result.status.clone(),
                    Err(err) => {
                        status = TestStatus::FAIL;
                    }
                }
                let title = self.render_test_title(
                    ui,
                    TitleType::Request,
                    name.to_string(),
                    status,
                );
                let collapsing = utils::open_collapsing(ui,title,|ui|{
                    match &result {
                        Ok(result) => {
                            for test_info in result.test_result.test_info_list.iter() {
                                let test_info_title = self.render_test_title(
                                    ui,
                                    TitleType::Assert,
                                    test_info.name.clone(), test_info.status.clone(),
                                );
                                ui.label(test_info_title);
                            }
                        }
                        Err(err) => {
                            ui.label(err.error.clone());
                        }
                    }
                });
                if collapsing.header_response.clicked(){
                    match &result {
                        Ok(result) => {
                            workspace_data.selected_test_run_result = Some(result.clone());
                        }
                        Err(err) => {
                            if err.response.is_some() {
                                workspace_data.selected_test_run_result = Some(TestRunResult {
                                    request: err.request.clone(),
                                    response: err.response.clone(),
                                    test_result: Default::default(),
                                    collection_path: err.collection_path.clone(),
                                    request_name: err.request_name.clone(),
                                    testcase: Default::default(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    fn render_result_tree(
        &mut self,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        egui::ScrollArea::vertical().max_height(ui.available_height()-30.0).show(ui,|ui|{
            if let Some(test_group_run_result) = self.test_group_run_result.clone() {
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
            }
        });
    }

    fn render_run_folder(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        collection_name: String,
        folder: &Rc<RefCell<CollectionFolder>>,
    ) {
        ui.add_enabled_ui(self.run_promise.is_none(), |ui| {
            if ui.button("Run Test").clicked() {
                let test_group_run_result = Arc::new(RwLock::new(TestGroupRunResults::default()));
                self.test_group_run_result = Some(test_group_run_result.clone());
                self.run_test_group(
                    workspace_data,
                    operation,
                    test_group_run_result,
                    collection_name,
                    folder.borrow().get_path(),
                    self.build_parent_testcase(),
                    folder.clone(),
                );
            }
        });
    }

    fn render_run_record(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        collection_name: String,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        record: Record,
    ) {
        ui.add_enabled_ui(self.run_promise.is_none(), |ui| {
            if ui.button("Run Test").clicked() {
                let test_group_run_result = Arc::new(RwLock::new(TestGroupRunResults::default()));
                self.test_group_run_result = Some(test_group_run_result.clone());
                self.run_test_record(
                    workspace_data,
                    operation,
                    test_group_run_result,
                    collection_name,
                    parent_folder.borrow().get_path(),
                    self.build_parent_testcase(),
                    record.clone(),
                );
            }
        });
    }
    fn render_manager_testcase(
        &mut self,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        test_item: TestItem,
    ) {
        ui.strong("Edit Self Testcase:");
        egui::ScrollArea::neither()
            .id_source("manager_testcase_scroll")
            .max_height(ui.available_height()-30.0)
            .show(ui, |ui| {
                self.manager_testcase_panel
                    .render(ui, workspace_data, test_item)
            });
    }
    fn render_select_testcase(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.strong("Select Parent Testcase:");
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
            TitleType::Testcase,
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
            TitleType::Group,
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
            TitleType::Request,
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
                                    TitleType::Assert,
                                    test_info.name.clone(),
                                    test_info.status.clone(),
                                );
                                ui.collapsing(test_info_title, |ui| {
                                    for tar in test_info.results.iter() {
                                        let test_assert_title = self.render_test_title(
                                            ui,
                                            TitleType::Assert,
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
                                TitleType::Assert,
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
                match r {
                    Ok(test_result) => {
                        workspace_data.selected_test_run_result = Some(test_result.clone())
                    }
                    Err(err) => {
                        if err.response.is_some() {
                            workspace_data.selected_test_run_result = Some(TestRunResult {
                                request: err.request.clone(),
                                response: err.response.clone(),
                                test_result: Default::default(),
                                collection_path: err.collection_path.clone(),
                                request_name: err.request_name.clone(),
                                testcase: Default::default(),
                            });
                        }
                    }
                }
            }
        };
    }

    fn render_test_title(&self, ui: &mut Ui,title_type:TitleType, name: String, status: TestStatus) -> LayoutJob {
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
            TestStatus::WAIT => rich_text = rich_text.background_color(Color32::DARK_BLUE),
            TestStatus::SKIP => rich_text = rich_text.background_color(Color32::GRAY),
        };
        rich_text.append_to(
            &mut request_test_result_name_layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
        RichText::new("  ").append_to(
            &mut request_test_result_name_layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
        let mut rich_text = RichText::new(title_type.to_string())
            .color(Color32::WHITE)
            .font(FontId {
                size: 12.0,
                family: FontFamily::Monospace,
            });
        match title_type {
            TitleType::Testcase => rich_text = rich_text.background_color(Color32::DARK_GRAY),
            TitleType::Request => rich_text = rich_text.background_color(Color32::GRAY),
            TitleType::Assert => rich_text = rich_text.color(Color32::DARK_GRAY).background_color(Color32::LIGHT_GRAY),
            TitleType::Group => rich_text = rich_text.background_color(Color32::BLACK),
        }
        rich_text.append_to(
            &mut request_test_result_name_layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
        RichText::new("  ").append_to(
            &mut request_test_result_name_layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
        RichText::new(name.as_str()).append_to(
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
        let script_tree = workspace_data.get_script_tree(collection_path.clone());
        self.run_promise = Some(operation.run_test_group_promise(
            envs,
            script_tree,
            test_group_run_result,
            collection_path,
            parent_testcase,
            folder,
        ));
    }

    fn run_test_record(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_name: String,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        record: Record,
    ) {
        let envs = workspace_data
            .get_build_envs(workspace_data.get_collection(Some(collection_name.clone())));
        let script_tree = workspace_data.get_script_tree(collection_path.clone());
        self.run_promise = Some(operation.run_test_record_promise(
            envs,
            script_tree,
            test_group_run_result,
            collection_path,
            parent_testcase,
            record,
        ));
    }
}
