use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{Align, Layout, ScrollArea, Ui};

use crate::data::{AppData, Collection, CollectionFolder, HttpRecord};
use crate::panels::{DataView, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct SaveWindows {
    save_windows_open: bool,
    save_windows_open2: bool,
    http_record: HttpRecord,
    select_collection_path: Option<String>,
    add_collection: bool,
    add_folder: bool,
    add_name: String,
}

impl SaveWindows {
    pub(crate) fn open(&mut self, http_record: HttpRecord) {
        self.save_windows_open = true;
        self.save_windows_open2 = true;
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
                utils::text_edit_singleline(ui, &mut self.http_record.name);
                ui.add_space(VERTICAL_GAP);
                ui.label("Request description (Optional)");
                utils::text_edit_multiline(ui, &mut self.http_record.desc);
                ui.add_space(VERTICAL_GAP);
                ui.label("Select a collection or folder to save to:");
                ui.add_space(VERTICAL_GAP);
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
                            if ui.link("◀ ".to_string() + name).clicked() {
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
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.add_name);
                            if ui.button("+").clicked() {
                                app_data.collections.insert_or_update(Collection {
                                    envs: Default::default(),
                                    folder: Rc::new(RefCell::new(CollectionFolder {
                                        name: self.add_name.clone(),
                                        desc: "".to_string(),
                                        auth: Default::default(),
                                        requests: Default::default(),
                                        folders: BTreeMap::default(),
                                    })),
                                });
                                self.add_name = "".to_string();
                                self.add_collection = false;
                            }
                        });
                    }
                    if self.add_folder {
                        self.add_collection = false;
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.add_name);
                            if ui.button("+").clicked() {
                                match &self.select_collection_path {
                                    None => {}
                                    Some(path) => {
                                        let (collection_name, option) = app_data
                                            .collections
                                            .get_mut_folder_with_path(path.clone());
                                        match option {
                                            None => {}
                                            Some(folder) => {
                                                folder.borrow_mut().folders.insert(
                                                    self.add_name.clone(),
                                                    Rc::new(RefCell::new(CollectionFolder {
                                                        name: self.add_name.to_string(),
                                                        desc: "".to_string(),
                                                        auth: Default::default(),
                                                        requests: Default::default(),
                                                        folders: Default::default(),
                                                    })),
                                                );
                                                app_data.collections.update(collection_name);
                                            }
                                        }
                                    }
                                }
                                self.add_name = "".to_string();
                                self.add_folder = false;
                            }
                        });
                    }
                    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        match self.select_collection_path.clone() {
                            None => {
                                for (name, collection) in app_data.collections.get_data().iter() {
                                    if utils::select_label(ui, name).clicked() {
                                        self.add_folder = false;
                                        self.add_collection = false;
                                        self.select_collection_path =
                                            Some(collection.folder.borrow().name.to_string());
                                    }
                                }
                            }
                            Some(path) => {
                                app_data
                                    .collections
                                    .get_mut_folder_with_path(path.clone())
                                    .1
                                    .map(|cf| {
                                        for (name, cf_child) in cf.borrow().folders.iter() {
                                            if utils::select_label(ui, name.clone()).clicked() {
                                                self.add_folder = false;
                                                self.add_collection = false;
                                                self.select_collection_path = Some(
                                                    path.clone()
                                                        + "/"
                                                        + cf_child.borrow().name.as_str(),
                                                )
                                            }
                                        }
                                        ui.set_enabled(false);
                                        for (_, hr) in cf.borrow().requests.iter() {
                                            utils::select_label(
                                                ui,
                                                utils::build_rest_ui_header(hr.clone(), ui),
                                            );
                                        }
                                    });
                            }
                        }
                    });
                    ui.add_space(VERTICAL_GAP);

                    egui::TopBottomPanel::bottom("save_bottom_panel")
                        .resizable(false)
                        .min_height(0.0)
                        .show_inside(ui, |ui| {
                            ui.add_space(VERTICAL_GAP);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| match &self
                                .select_collection_path
                            {
                                None => {
                                    ui.set_enabled(false);
                                    ui.button("Save");
                                    ui.set_enabled(true);
                                }
                                Some(path) => {
                                    if ui
                                        .button(
                                            "Save to ".to_string()
                                                + path.split("/").last().unwrap(),
                                        )
                                        .clicked()
                                    {
                                        let (collection_name, option) = app_data
                                            .collections
                                            .get_mut_folder_with_path(path.clone());
                                        match option {
                                            None => {}
                                            Some(cf) => {
                                                cf.borrow_mut().requests.insert(
                                                    self.http_record.name.clone(),
                                                    self.http_record.clone(),
                                                );
                                                app_data.collections.update(collection_name);
                                            }
                                        }
                                        self.save_windows_open2 = false;
                                    }
                                    if ui.button("Cancel").clicked() {
                                        self.save_windows_open2 = false;
                                    }
                                }
                            });
                        });
                });
            });
        if !self.save_windows_open2 {
            self.save_windows_open = false;
        }
    }
}
