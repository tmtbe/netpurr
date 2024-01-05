use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{Align, Button, Layout, ScrollArea, Ui};

use crate::data::{Collection, CollectionFolder, HttpRecord, WorkspaceData};
use crate::operation::Operation;
use crate::panels::{DataView, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct SaveWindows {
    save_windows_open: bool,
    http_record: HttpRecord,
    select_collection_path: Option<String>,
    add_collection: bool,
    add_folder: bool,
    add_name: String,
    title: String,
    id: String,
    edit: bool,
    old_name: String,
}

impl SaveWindows {
    pub(crate) fn open(
        &mut self,
        http_record: HttpRecord,
        default_path: Option<String>,
        edit: bool,
    ) {
        self.save_windows_open = true;
        self.http_record = http_record;
        self.old_name = self.http_record.name.clone();
        if self.http_record.name == "" {
            self.http_record.name = self.http_record.request.base_url.clone();
        } else {
            self.http_record.name = self.http_record.name.clone();
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
                    for (name, collection) in workspace_data.collections.get_data().iter() {
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
                        .collections
                        .get_mut_folder_with_path(path.clone())
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
                    let (_, option) = workspace_data
                        .collections
                        .get_folder_with_path(path.clone());
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
                        let (collection_name, option) = workspace_data
                            .collections
                            .get_mut_folder_with_path(path.clone());
                        match option {
                            None => {}
                            Some(folder) => {
                                workspace_data.collections.insert_folder(
                                    folder.clone(),
                                    Rc::new(RefCell::new(CollectionFolder {
                                        name: self.add_name.to_string(),
                                        parent_path: path.clone(),
                                        desc: "".to_string(),
                                        auth: Default::default(),
                                        is_root: false,
                                        requests: Default::default(),
                                        folders: Default::default(),
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
                .collections
                .get_data()
                .contains_key(self.add_name.as_str())
            {
                ui.add_enabled(false, Button::new("+"));
            } else {
                if ui.button("+").clicked() {
                    workspace_data.collections.insert_collection(Collection {
                        envs: Default::default(),
                        folder: Rc::new(RefCell::new(CollectionFolder {
                            name: self.add_name.clone(),
                            parent_path: ".".to_string(),
                            desc: "".to_string(),
                            auth: Default::default(),
                            is_root: true,
                            requests: Default::default(),
                            folders: BTreeMap::default(),
                        })),
                        script: "".to_string(),
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
                        Some(path) => {
                            let mut ui_enable = true;
                            let button_name =
                                "Save to ".to_string() + path.split("/").last().unwrap();
                            let (collection_name, option) = workspace_data
                                .collections
                                .get_mut_folder_with_path(path.clone());
                            match &option {
                                None => {}
                                Some(cf) => {
                                    if self.edit && self.old_name == self.http_record.name {
                                        ui_enable = true;
                                    } else {
                                        if cf
                                            .borrow()
                                            .requests
                                            .contains_key(self.http_record.name.as_str())
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
                                                cf.borrow_mut()
                                                    .requests
                                                    .remove(self.old_name.as_str());
                                            }
                                            workspace_data.collections.insert_http_record(
                                                cf.clone(),
                                                self.http_record.clone(),
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

impl DataView for SaveWindows {
    type CursorType = i32;

    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        let mut save_windows_open = self.save_windows_open;
        operation.lock_ui(
            "save_".to_string() + self.id.as_str(),
            self.save_windows_open,
        );
        egui::Window::new(self.title.clone())
            .default_open(true)
            .max_width(500.0)
            .default_height(400.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut save_windows_open)
            .show(ui.ctx(), |ui| {
                ui.label("Requests in Poster are saved in collections (a group of requests).");
                ui.add_space(VERTICAL_GAP);
                ui.label("Request name");
                utils::text_edit_singleline_filter_justify(ui, &mut self.http_record.name);
                ui.add_space(VERTICAL_GAP);
                ui.label("Request description (Optional)");
                utils::text_edit_multiline_justify(ui, &mut self.http_record.desc);
                ui.add_space(VERTICAL_GAP);
                if !self.edit {
                    ui.label("Select a collection or folder to save to:");
                    ui.add_space(VERTICAL_GAP);
                    self.render(workspace_data, ui);
                    ui.add_space(VERTICAL_GAP);
                }
                self.render_save_bottom_panel(workspace_data, ui);
            });
        if !save_windows_open {
            self.save_windows_open = save_windows_open;
        }
    }
}
