use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{ScrollArea, Ui};

use crate::data::{AppData, Collection, CollectionFolder, HttpRecord};
use crate::panels::{DataView, VERTICAL_GAP};

#[derive(Default)]
pub struct SaveWindows {
    save_windows_open: bool,
    http_record: HttpRecord,
    select_collection_path: Option<String>,
    add_collection: bool,
    add_folder: bool,
    add_name: String,
}

impl SaveWindows {
    pub(crate) fn open(&mut self, http_record: HttpRecord) {
        self.save_windows_open = true;
        self.http_record = http_record;
        self.http_record.name = self.http_record.request.base_url.clone();
        self.add_folder = false;
        self.add_collection = false;
        self.add_name = "".to_string();
    }
}

impl DataView for SaveWindows {
    type CursorType = i32;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        app_data.lock_ui("save".to_string(), self.save_windows_open);
        egui::Window::new("SAVE REQUEST")
            .default_open(true)
            .default_width(500.0)
            .default_height(300.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut self.save_windows_open)
            .show(ui.ctx(), |ui| {
                ui.label("Requests in Poster are saved in collections (a group of requests).");
                ui.add_space(VERTICAL_GAP);
                ui.label("Request name");
                ui.text_edit_singleline(&mut self.http_record.name);
                ui.add_space(VERTICAL_GAP);
                ui.label("Request description (Optional)");
                ui.text_edit_multiline(&mut self.http_record.desc);
                ui.add_space(VERTICAL_GAP);
                ui.label("Select a collection or folder to save to:");

                ui.vertical(|ui| {
                    ui.horizontal(|ui| match &self.select_collection_path {
                        None => {
                            ui.label("All Collections");
                            if ui.link("+ Create Collection").clicked() {
                                self.add_collection = true;
                            };
                        }
                        Some(name) => {
                            ui.link(name);
                            if ui.link("+ Create Folder").clicked() {
                                self.add_folder = true;
                            };
                        }
                    });
                    if self.add_collection {
                        if ui.text_edit_singleline(&mut self.add_name).double_clicked() {
                            app_data.collections.insert_or_update(Collection {
                                envs: Default::default(),
                                folder: Rc::new(RefCell::new(CollectionFolder {
                                    parent: None,
                                    name: self.add_name.clone(),
                                    desc: "".to_string(),
                                    auth: Default::default(),
                                    requests: vec![],
                                    folders: BTreeMap::default(),
                                })),
                            });
                            self.add_name = "".to_string();
                            self.add_collection = false;
                        }
                    }
                    if self.add_folder {
                        if ui.text_edit_singleline(&mut self.add_name).double_clicked() {
                            match &self.select_collection_path {
                                None => {}
                                Some(path) => {
                                    match app_data
                                        .collections
                                        .get_mut_folder_with_path(path.clone())
                                    {
                                        None => {}
                                        Some(folder) => {
                                            folder.borrow_mut().folders.insert(
                                                self.add_name.clone(),
                                                Rc::new(RefCell::new(CollectionFolder {
                                                    parent: Some(folder.clone()),
                                                    name: self.add_name.to_string(),
                                                    desc: "".to_string(),
                                                    auth: Default::default(),
                                                    requests: vec![],
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
                    }
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        match self.select_collection_path.clone() {
                            None => {
                                for (name, collection) in app_data.collections.get_data().iter() {
                                    if ui.selectable_label(false, name).clicked() {
                                        self.select_collection_path =
                                            Some(collection.folder.borrow().name.to_string());
                                    }
                                }
                            }
                            Some(path) => {
                                app_data
                                    .collections
                                    .get_mut_folder_with_path(path.clone())
                                    .map(|cf| {
                                        for (name, cf_child) in cf.borrow().folders.iter() {
                                            if ui.selectable_label(false, name.clone()).clicked() {
                                                self.select_collection_path = Some(
                                                    path.clone()
                                                        + "/"
                                                        + cf_child.borrow().name.as_str(),
                                                )
                                            }
                                        }
                                        ui.set_enabled(false);
                                        for hr in cf.borrow().requests.iter() {
                                            ui.selectable_label(false, hr.name.clone());
                                        }
                                    });
                            }
                        }
                    });
                });
            });
    }
}
