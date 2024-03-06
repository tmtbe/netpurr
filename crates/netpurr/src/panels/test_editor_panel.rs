use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use egui::text::LayoutJob;
use egui::{Align, FontSelection, RichText, Style, Ui};
use poll_promise::Promise;

use netpurr_core::data::collections::CollectionFolder;
use netpurr_core::runner::{RunRequestInfo, TestGroupRunResults, TestRunError, TestRunResult};
use netpurr_core::script::ScriptScope;

use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;

#[derive(Default)]
pub struct TestEditorPanel {
    test_group_run_result: Option<Arc<Mutex<TestGroupRunResults>>>,
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
                                Arc::new(Mutex::new(TestGroupRunResults::default()));
                            self.test_group_run_result = Some(test_group_run_result.clone());
                            self.run_test_group(
                                workspace_data,
                                operation,
                                test_group_run_result,
                                collection_name,
                                collection_path,
                                folder,
                            );
                        }
                    });
                }
            });
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 30.0)
            .show(ui, |ui| {
                let result_tree = self.build_tree();
                for (name, rf) in result_tree.folders.iter() {
                    ui.collapsing(name, |ui| {
                        for result in rf.results.iter() {
                            match result {
                                Ok(test_run_result) => {
                                    let style = Style::default();
                                    let mut request_test_result_name_layout_job =
                                        LayoutJob::default();
                                    RichText::new(
                                        test_run_result.test_result.status.clone().to_string(),
                                    )
                                    .background_color(ui.visuals().extreme_bg_color)
                                    .append_to(
                                        &mut request_test_result_name_layout_job,
                                        &style,
                                        FontSelection::Default,
                                        Align::Center,
                                    );
                                    RichText::new(
                                        "  ".to_string() + test_run_result.request_name.as_str(),
                                    )
                                    .append_to(
                                        &mut request_test_result_name_layout_job,
                                        &style,
                                        FontSelection::Default,
                                        Align::Center,
                                    );
                                    if ui
                                        .collapsing(request_test_result_name_layout_job, |ui| {
                                            for test_info in
                                                test_run_result.test_result.test_info_list.iter()
                                            {
                                                ui.collapsing(test_info.name.clone(), |ui| {
                                                    for test_assert_result in
                                                        test_info.results.iter()
                                                    {
                                                        ui.label(
                                                            test_assert_result
                                                                .assert_result
                                                                .clone()
                                                                .to_string(),
                                                        );
                                                        ui.label(test_assert_result.msg.clone());
                                                    }
                                                });
                                            }
                                        })
                                        .header_response
                                        .clicked()
                                    {
                                        workspace_data.selected_test_run_result =
                                            Some(test_run_result.clone())
                                    };
                                }
                                Err(err) => {
                                    ui.label(err.error.clone());
                                }
                            };
                        }
                    });
                }
            });
    }

    fn run_test_group(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        test_group_run_result: Arc<Mutex<TestGroupRunResults>>,
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

    fn build_tree(&self) -> ResultTree {
        let mut result_tree = ResultTree::default();
        if let Some(tg) = &self.test_group_run_result {
            if let Ok(t) = tg.try_lock() {
                for result in t.results.iter().cloned() {
                    match &result {
                        Ok(s) => {
                            let path = s.collection_path.clone().unwrap_or_default();
                            if !result_tree.folders.contains_key(path.as_str()) {
                                result_tree
                                    .folders
                                    .insert(path.clone(), ResultTreeFolder::default());
                            }
                            result_tree
                                .folders
                                .get_mut(path.as_str())
                                .unwrap()
                                .results
                                .push(result.clone());
                        }
                        Err(e) => {
                            let path = e.collection_path.clone().unwrap_or_default();
                            if !result_tree.folders.contains_key(path.as_str()) {
                                result_tree
                                    .folders
                                    .insert(path.clone(), ResultTreeFolder::default());
                            }
                            result_tree
                                .folders
                                .get_mut(path.as_str())
                                .unwrap()
                                .results
                                .push(result.clone());
                        }
                    }
                }
            }
        }
        return result_tree;
    }
}

#[derive(Default)]
struct ResultTree {
    folders: BTreeMap<String, ResultTreeFolder>,
}

#[derive(Default)]
struct ResultTreeFolder {
    results: Vec<Result<TestRunResult, TestRunError>>,
}
