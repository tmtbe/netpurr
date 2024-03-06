use egui::{ScrollArea, Ui};

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
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
            workspace_data.selected_test_group_path = None
        }
        workspace_data
            .get_collection_by_name(self.collection_name.clone())
            .map(|collection| {
                let mut name = "".to_string();
                match &workspace_data.selected_test_group_path {
                    None => {
                        name = collection.folder.borrow().name.clone();
                        workspace_data.selected_test_group_path =
                            Some(collection.folder.borrow().name.clone());
                    }
                    Some(path) => {
                        name = path.clone();
                    }
                }
                ui.label("Select Test Group");
                if ui.link("◀ ".to_string() + name.as_str()).clicked() {
                    let paths: Vec<&str> = name.split("/").collect();
                    if paths.len() > 1 {
                        let new_paths = &paths[0..paths.len() - 1];
                        workspace_data.selected_test_group_path = Some(new_paths.join("/"));
                    }
                }
                ui.separator();
                self.render_list(workspace_data, ui);
            });
    }

    fn render_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ScrollArea::vertical()
            .max_height(ui.available_height() - 30.0)
            .show(ui, |ui| {
                match workspace_data.selected_test_group_path.clone() {
                    None => {}
                    Some(path) => {
                        workspace_data
                            .get_folder_with_path(path.clone())
                            .1
                            .map(|cf| {
                                for (name, cf_child) in cf.borrow().folders.iter() {
                                    if utils::select_label(ui, name.clone()).clicked() {
                                        workspace_data.selected_test_group_path = Some(
                                            path.clone() + "/" + cf_child.borrow().name.as_str(),
                                        );
                                    }
                                }
                                ui.set_enabled(false);
                                for (_, hr) in cf.borrow().requests.iter() {
                                    utils::select_label(
                                        ui,
                                        utils::build_rest_ui_header(hr.clone(), None, ui),
                                    );
                                }
                            });
                    }
                }
            });
    }
}
