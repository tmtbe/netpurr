use egui::{ScrollArea, Ui};
use netpurr_core::data::collections::CollectionFolder;
use std::cell::RefCell;
use std::rc::Rc;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::{TestItem, WorkspaceData};
use crate::operation::operation::Operation;
use crate::utils;

#[derive(Default)]
pub struct TestGroupPanel {
    collection_name: String,
}
impl TestGroupPanel {
    pub fn render(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
        ui: &mut Ui,
    ) {
        let collection_name = config_data.select_collection().unwrap_or_default();
        if self.collection_name != collection_name {
            self.collection_name = collection_name.clone();
            workspace_data.selected_test_item = None
        }
        workspace_data
            .get_collection_by_name(self.collection_name.clone())
            .map(|collection| {
                let mut name = "".to_string();
                match &workspace_data.selected_test_item {
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
                if ui.link("◀ ".to_string() + name.as_str()).clicked() {
                    let paths: Vec<&str> = name.split("/").collect();
                    if paths.len() > 1 {
                        let new_paths = &paths[0..paths.len() - 1];
                        if let (_, Some(folder)) =
                            workspace_data.get_folder_with_path(new_paths.join("/"))
                        {
                            workspace_data.selected_test_item = Some(TestItem::Folder(
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
            .show(ui, |ui| match workspace_data.selected_test_item.clone() {
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
        &self,
        workspace_data: &mut WorkspaceData,
        ui: &mut Ui,
        collection_name: String,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        for (name, cf_child) in folder.borrow().folders.iter() {
            if utils::select_label(ui, name.clone()).clicked() {
                workspace_data.selected_test_item =
                    Some(TestItem::Folder(collection_name.clone(), cf_child.clone()));
            }
        }
        for (_, hr) in folder.borrow().requests.iter() {
            if utils::select_label(ui, utils::build_rest_ui_header(hr.clone(), None, ui)).clicked()
            {
                workspace_data.selected_test_item = Some(TestItem::Record(
                    collection_name.clone(),
                    folder.clone(),
                    hr.name(),
                ));
            }
        }
    }
}
