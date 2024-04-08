use std::cell::RefCell;
use std::fmt::format;
use std::rc::Rc;

use egui::{ScrollArea, Ui};

use netpurr_core::data::collections::CollectionFolder;
use netpurr_core::data::workspace_data::{TestItem, WorkspaceData};

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::utils;

#[derive(Default)]
pub struct TestGroupPanel {
    collection_name: String,
    selected_test_item_name: String,
    selected_test_item: Option<TestItem>,
}
impl TestGroupPanel {
    pub fn render(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
        ui: &mut Ui,
    ) {
        if workspace_data.selected_test_item.is_none(){
            self.selected_test_item = None;
        }
        if self.selected_test_item.is_none() {
            self.selected_test_item = workspace_data.selected_test_item.clone();
        }
        let collection_name = config_data.select_collection().unwrap_or_default();
        if self.collection_name != collection_name {
            self.collection_name = collection_name.clone();
            workspace_data.selected_test_item = None
        }
        workspace_data
            .get_collection_by_name(self.collection_name.clone())
            .map(|collection| {
                let mut name = "".to_string();
                match &self.selected_test_item {
                    None => {
                        name = collection.folder.borrow().name.clone();
                        workspace_data.selected_test_item = Some(TestItem::Folder(
                            self.collection_name.clone(),
                            collection.folder.clone(),
                        ));
                    }
                    Some(test_item) => match test_item {
                        TestItem::Folder(_, f) => {
                            name = f.borrow().get_path();
                        }
                        TestItem::Record(_, f, _) => {
                            name = f.borrow().get_path();
                        }
                    },
                }
                ui.label("Select Test Group");
                if ui.link(format!("{} {} {}",egui_phosphor::regular::ARROW_LEFT,egui_phosphor::regular::FOLDER, name)).clicked() {
                    let paths: Vec<&str> = name.split("/").collect();
                    if paths.len() > 1 {
                        let new_paths = &paths[0..paths.len() - 1];
                        if let (_, Some(folder)) =
                            workspace_data.get_folder_with_path(new_paths.join("/"))
                        {
                            self.selected_test_item = Some(TestItem::Folder(
                                self.collection_name.clone(),
                                folder.clone(),
                            ));
                        }
                    }
                }
                ui.separator();
                self.render_list(workspace_data, ui);
            });
    }

    fn render_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ScrollArea::vertical()
            .max_height(ui.available_height() - 30.0)
            .show(ui, |ui| match self.selected_test_item.clone() {
                None => {}
                Some(test_item) => match test_item {
                    TestItem::Folder(collection_name, folder) => {
                        self.render_folder(workspace_data, ui, collection_name, folder);
                    }
                    TestItem::Record(collection_name, folder, _) => {
                        self.render_folder(workspace_data, ui, collection_name, folder);
                    }
                },
            });
    }

    fn render_folder(
        &mut self,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        collection_name: String,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        let label = utils::select_value(
            ui,
            &mut self.selected_test_item_name,
            folder.borrow().get_path(),
            format!("{} ../{}",egui_phosphor::regular::FOLDER,folder.borrow().name.clone()),
        );
        if label.clicked() {
            workspace_data.selected_test_item =
                Some(TestItem::Folder(collection_name.clone(), folder.clone()));
        }
        if label.double_clicked() {
            let path = folder.borrow().get_path();
            let count = path.split("/").count();
            if count>1 {
                let parent_paths: Vec<&str> = path.split("/").take(count - 1).collect();
                let parent_path = parent_paths.join("/");
                let (_, parent_folder) = workspace_data.get_folder_with_path(parent_path);
                parent_folder.map(|p| {
                    self.selected_test_item =
                        Some(TestItem::Folder(collection_name.clone(), p.clone()));
                });
            }
        }
        for (name, cf_child) in folder.borrow().folders.iter() {
            let label = utils::select_value(
                ui,
                &mut self.selected_test_item_name,
                cf_child.borrow().get_path(),
                format!("{} {}",egui_phosphor::regular::FOLDER,name.clone()),
            );
            if label.clicked() {
                workspace_data.selected_test_item =
                    Some(TestItem::Folder(collection_name.clone(), cf_child.clone()));
            }
            if label.double_clicked() {
                self.selected_test_item =
                    Some(TestItem::Folder(collection_name.clone(), cf_child.clone()));
            }
        }
        for (_, hr) in folder.borrow().requests.iter() {
            let label = utils::select_value(
                ui,
                &mut self.selected_test_item_name,
                format!("{}/{}", folder.borrow().get_path(), hr.name()),
                utils::build_rest_ui_header(hr.clone(), None, ui),
            );
            if label.clicked() {
                workspace_data.selected_test_item = Some(TestItem::Record(
                    collection_name.clone(),
                    folder.clone(),
                    hr.name(),
                ));
            }
            if label.double_clicked() {
                self.selected_test_item = Some(TestItem::Record(
                    collection_name.clone(),
                    folder.clone(),
                    hr.name(),
                ));
            }
        }
    }
}
