use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{Align, Button, Layout, ScrollArea, Ui};

use netpurr_core::data::collections::{Collection, CollectionFolder};
use netpurr_core::data::record::Record;
use netpurr_core::data::workspace_data::WorkspaceData;

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::VERTICAL_GAP;
use crate::utils;

#[derive(Default)]
pub struct SaveWindows {
    save_windows_open: bool,
    record: Record,
    select_collection_path: Option<String>,
    add_collection: bool,
    add_folder: bool,
    add_name: String,
    title: String,
    id: String,
    edit: bool,
    old_name: String,
}

impl Window for SaveWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new(self.title.clone())
            .max_width(500.0)
            .default_height(400.0)
            .collapsible(false)
            .resizable(true)
    }

    fn set_open(&mut self, open: bool) {
        self.save_windows_open = open;
    }

    fn get_open(&self) -> bool {
        self.save_windows_open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        _: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        ui.label("Requests in Netpurr are saved in collections (a group of requests).");
        ui.add_space(VERTICAL_GAP);
        ui.label("Request name");
        utils::text_edit_singleline_filter_justify(ui, &mut self.record.name());
        ui.add_space(VERTICAL_GAP);
        ui.label("Request description (Optional)");
        utils::text_edit_multiline_justify(ui, &mut self.record.desc());
        ui.add_space(VERTICAL_GAP);
        if !self.edit {
            ui.label("Select a collection or folder to save to:");
            ui.add_space(VERTICAL_GAP);
            self.render(workspace_data, ui);
            ui.add_space(VERTICAL_GAP);
        }
        self.render_save_bottom_panel(workspace_data, ui);
    }
}

impl SaveWindows {
    pub fn with(mut self, record: Record, default_path: Option<String>, edit: bool) -> Self {
        self.save_windows_open = true;
        self.record = record;
        self.old_name = self.record.name();
        if self.record.name() == "" {
            self.record.set_name(self.record.base_url());
        } else {
            self.record.set_name(self.record.name());
        }
        self.add_folder = false;
        self.add_collection = false;
        self.add_name = "".to_string();
        self.edit = edit;
        match &default_path {
            None => {
                self.title = "SAVE REQUEST".to_string();
            }
            Some(_) => {
                if !edit {
                    self.title = "SAVE AS REQUEST".to_string();
                } else {
                    self.title = "EDIT REQUEST".to_string();
                }
            }
        }
        self.select_collection_path = default_path.clone();
        self.id = default_path.clone().unwrap_or("new".to_string());
        self
    }

    fn render(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| match &self.select_collection_path {
                None => {
                    ui.label("All Collections");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.link("+ Create Collection").clicked() {
                            self.add_collection = true;
                        };
                    });
                }
                Some(name) => {
                    if ui.link("◀ ".to_string() + name.as_str()).clicked() {
                        self.add_folder = false;
                        self.add_collection = false;
                        let paths: Vec<&str> = name.split("/").collect();
                        if paths.len() == 1 {
                            self.select_collection_path = None;
                        } else {
                            let new_paths = &paths[0..paths.len() - 1];
                            self.select_collection_path = Some(new_paths.join("/"));
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.link("+ Create Folder").clicked() {
                            self.add_folder = true;
                        };
                    });
                }
            });
            ui.add_space(VERTICAL_GAP);
            if self.add_collection {
                self.add_folder = false;
                self.render_add_collection(workspace_data, ui);
            }
            if self.add_folder {
                self.add_collection = false;
                self.render_add_folder(workspace_data, ui);
            }
            self.render_list(workspace_data, ui);
        });
    }

    fn render_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            match self.select_collection_path.clone() {
                None => {
                    for (name, collection) in workspace_data.get_collections().iter() {
                        if utils::select_label(ui, name).clicked() {
                            self.add_folder = false;
                            self.add_collection = false;
                            self.select_collection_path =
                                Some(collection.folder.borrow().name.to_string());
                        }
                    }
                }
                Some(path) => {
                    workspace_data
                        .get_folder_with_path(path.clone())
                        .1
                        .map(|cf| {
                            for (name, cf_child) in cf.borrow().folders.iter() {
                                if utils::select_label(ui, name.clone()).clicked() {
                                    self.add_folder = false;
                                    self.add_collection = false;
                                    self.select_collection_path =
                                        Some(path.clone() + "/" + cf_child.borrow().name.as_str())
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

    fn render_add_folder(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.horizontal(|ui| {
            match &self.select_collection_path {
                None => {}
                Some(path) => {
                    let (_, option) = workspace_data.get_folder_with_path(path.clone());
                    match option {
                        None => {}
                        Some(folder) => {
                            if folder.borrow().folders.contains_key(self.add_name.as_str()) {
                                ui.set_enabled(false);
                            }
                        }
                    }
                }
            }
            if ui.button("+").clicked() {
                match &self.select_collection_path {
                    None => {}
                    Some(path) => {
                        let (collection_name, option) =
                            workspace_data.get_folder_with_path(path.clone());
                        match option {
                            None => {}
                            Some(folder) => {
                                workspace_data.collection_insert_folder(
                                    folder.clone(),
                                    Rc::new(RefCell::new(CollectionFolder {
                                        name: self.add_name.to_string(),
                                        parent_path: path.clone(),
                                        desc: "".to_string(),
                                        auth: Default::default(),
                                        is_root: false,
                                        requests: Default::default(),
                                        folders: Default::default(),
                                        pre_request_script: "".to_string(),
                                        test_script: "".to_string(),
                                        testcases: Default::default(),
                                    })),
                                );
                            }
                        }
                    }
                }
                self.add_name = "".to_string();
                self.add_folder = false;
            }
            utils::text_edit_singleline_filter_justify(ui, &mut self.add_name);
        });
    }

    fn render_add_collection(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if workspace_data
                .get_collections()
                .contains_key(self.add_name.as_str())
            {
                ui.add_enabled(false, Button::new("+"));
            } else {
                if ui.button("+").clicked() {
                    workspace_data.add_collection(Collection {
                        folder: Rc::new(RefCell::new(CollectionFolder {
                            name: self.add_name.clone(),
                            parent_path: ".".to_string(),
                            desc: "".to_string(),
                            auth: Default::default(),
                            is_root: true,
                            requests: Default::default(),
                            folders: BTreeMap::default(),
                            pre_request_script: "".to_string(),
                            test_script: "".to_string(),
                            testcases: Default::default(),
                        })),
                        ..Default::default()
                    });
                    self.add_name = "".to_string();
                    self.add_collection = false;
                }
            }
            utils::text_edit_singleline_filter_justify(ui, &mut self.add_name);
        });
    }

    fn render_save_bottom_panel(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("save_bottom_panel_".to_string() + self.id.as_str())
            .resizable(false)
            .min_height(0.0)
            .show_inside(ui, |ui| {
                ui.add_space(VERTICAL_GAP);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    match &self.select_collection_path {
                        None => {
                            ui.add_enabled_ui(false, |ui| {
                                ui.button("Save");
                            });
                        }
                        Some(collection_path) => {
                            let mut ui_enable = true;
                            if self.record.name().is_empty() {
                                ui_enable = false;
                            }
                            let button_name = "Save to ".to_string()
                                + collection_path.split("/").last().unwrap_or_default();
                            let (_, option) =
                                workspace_data.get_folder_with_path(collection_path.clone());
                            match &option {
                                None => {}
                                Some(cf) => {
                                    if self.edit && self.old_name == self.record.name() {
                                        ui_enable = true;
                                    } else {
                                        if cf
                                            .borrow()
                                            .requests
                                            .contains_key(self.record.name().as_str())
                                        {
                                            ui_enable = false;
                                        }
                                    }
                                }
                            }
                            ui.add_enabled_ui(ui_enable, |ui| {
                                if ui.button(button_name).clicked() {
                                    match &option {
                                        None => {}
                                        Some(cf) => {
                                            if self.edit {
                                                workspace_data.collection_remove_http_record(
                                                    cf.clone(),
                                                    self.old_name.clone(),
                                                )
                                            }
                                            workspace_data.collection_insert_record(
                                                cf.clone(),
                                                self.record.clone(),
                                            );
                                            workspace_data.update_crt_old_name_to_new_name(
                                                collection_path.clone(),
                                                self.old_name.clone(),
                                                self.record.name(),
                                            );
                                        }
                                    }
                                    self.save_windows_open = false;
                                }
                            });

                            if ui.button("Cancel").clicked() {
                                self.save_windows_open = false;
                            }
                        }
                    }
                });
            });
    }
}
