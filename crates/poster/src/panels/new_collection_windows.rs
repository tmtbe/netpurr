use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;

use egui::{Align, Button, Checkbox, Layout, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{AppData, Collection, CollectionFolder, EnvironmentItem};
use crate::panels::auth_panel::AuthPanel;
use crate::panels::{AlongDataView, DataView, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct NewCollectionWindows {
    title_name: String,
    new_select_env_item: EnvironmentItem,
    new_collection_windows_open: bool,
    new_collection: Collection,
    old_collection_name: Option<String>,
    old_folder_name: Option<String>,
    folder: Rc<RefCell<CollectionFolder>>,
    parent_folder: Option<Rc<RefCell<CollectionFolder>>>,
    new_collection_content_type: NewCollectionContentType,
    auth_panel: AuthPanel,
}

#[derive(Clone, EnumString, EnumIter, PartialEq, Display)]
enum NewCollectionContentType {
    Description,
    Authorization,
    Variables,
}

impl Default for NewCollectionContentType {
    fn default() -> Self {
        NewCollectionContentType::Description
    }
}

impl NewCollectionWindows {
    pub(crate) fn open_collection(&mut self, collection: Option<Collection>) {
        self.new_collection_windows_open = true;
        match collection {
            None => {
                self.new_collection = Collection::default();
                self.title_name = "CREATE A NEW COLLECTION".to_string();
            }
            Some(collection) => {
                self.new_collection = collection;
                self.old_collection_name = Some(self.new_collection.folder.borrow().name.clone());
                self.title_name = "EDIT COLLECTION".to_string();
            }
        }

        self.new_collection_content_type = NewCollectionContentType::Description;
        self.folder = self.new_collection.folder.clone();
        self.parent_folder = None;
    }
    pub(crate) fn open_folder(
        &mut self,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Option<Rc<RefCell<CollectionFolder>>>,
    ) {
        self.new_collection_windows_open = true;
        match folder {
            None => {
                self.new_collection = collection;
                self.title_name = "CREATE A NEW FOLDER".to_string();
                self.folder = Rc::new(RefCell::new(CollectionFolder {
                    name: "".to_string(),
                    desc: "".to_string(),
                    auth: Default::default(),
                    is_root: false,
                    requests: Default::default(),
                    folders: Default::default(),
                }));
            }
            Some(cf) => {
                self.new_collection = collection;
                self.old_folder_name = Some(cf.borrow().name.clone());
                self.title_name = "EDIT FOLDER".to_string();
                self.folder = Rc::new(RefCell::new(CollectionFolder {
                    name: cf.borrow().name.clone(),
                    desc: cf.borrow().desc.clone(),
                    auth: cf.borrow().auth.clone(),
                    is_root: cf.borrow().is_root,
                    requests: cf.borrow().requests.clone(),
                    folders: cf.borrow().folders.clone(),
                }));
            }
        }
        self.parent_folder = Some(parent_folder.clone());
        self.new_collection_content_type = NewCollectionContentType::Description;
    }

    fn build_variables(&mut self, ui: &mut Ui) {
        ui.label("These variables are specific to this collection and its requests. ");
        ui.add_space(VERTICAL_GAP);
        ui.separator();
        ui.add_space(VERTICAL_GAP);
        let mut delete_index = None;
        ui.push_id("new_collection_environment_table", |ui| {
            let table = TableBuilder::new(ui)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto())
                .column(Column::exact(20.0))
                .column(Column::initial(200.0).range(40.0..=300.0))
                .column(Column::remainder())
                .max_scroll_height(400.0);
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("VARIABLE");
                    });
                    header.col(|ui| {
                        ui.strong("VALUE");
                    });
                })
                .body(|mut body| {
                    for (index, item) in self.new_collection.envs.items.iter_mut().enumerate() {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut item.enable, "");
                            });
                            row.col(|ui| {
                                if ui.button("x").clicked() {
                                    delete_index = Some(index)
                                }
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut item.key);
                            });
                            row.col(|ui| {
                                TextEdit::singleline(&mut item.value)
                                    .desired_width(f32::INFINITY)
                                    .ui(ui);
                            });
                        });
                    }
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.add_enabled(
                                false,
                                Checkbox::new(&mut self.new_select_env_item.enable, ""),
                            );
                        });
                        row.col(|ui| {
                            ui.add_enabled(false, Button::new("x"));
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut self.new_select_env_item.key);
                        });
                        row.col(|ui| {
                            TextEdit::singleline(&mut self.new_select_env_item.value)
                                .desired_width(f32::INFINITY)
                                .ui(ui);
                        });
                    });
                });
        });
        if delete_index.is_some() {
            self.new_collection.envs.items.remove(delete_index.unwrap());
        }
        if self.new_select_env_item.key != "" || self.new_select_env_item.value != "" {
            self.new_select_env_item.enable = true;
            self.new_collection
                .envs
                .items
                .push(self.new_select_env_item.clone());
            self.new_select_env_item.key = "".to_string();
            self.new_select_env_item.value = "".to_string();
            self.new_select_env_item.enable = false;
        }
    }

    fn build_auth(&mut self, ui: &mut Ui) {
        ui.label("This authorization method will be used for every request in this collection. You can override this by specifying one in the request.");
        ui.add_space(VERTICAL_GAP);
        ui.separator();
        ui.add_space(VERTICAL_GAP);
        self.auth_panel
            .set_collection_folder(self.new_collection.clone(), self.folder.clone());
        self.auth_panel
            .set_and_render(ui, &mut self.folder.borrow_mut().auth);
    }

    fn build_desc(&mut self, ui: &mut Ui) {
        ui.label("This description will show in your collection’s documentation, along with the descriptions of its folders and requests.");
        ui.add_space(VERTICAL_GAP);
        ui.separator();
        ui.add_space(VERTICAL_GAP);
        utils::text_edit_multiline(ui, &mut self.folder.borrow_mut().desc);
        ui.add_space(VERTICAL_GAP);
    }

    fn bottom_panel(&mut self, app_data: &mut AppData, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("new_collection_bottom_panel")
            .resizable(false)
            .min_height(0.0)
            .show_inside(ui, |ui| {
                ui.add_space(VERTICAL_GAP);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        self.new_collection_windows_open = false;
                    }
                    if self.new_collection.folder.borrow().name != "" {
                        match &self.parent_folder {
                            None => match &self.old_collection_name {
                                None => {
                                    if app_data
                                        .collections
                                        .get_data()
                                        .contains_key(self.folder.borrow().name.as_str())
                                    {
                                        ui.set_enabled(false)
                                    }
                                }
                                Some(old_name) => {
                                    if old_name != self.folder.borrow().name.as_str()
                                        && app_data
                                            .collections
                                            .get_data()
                                            .contains_key(self.folder.borrow().name.as_str())
                                    {
                                        ui.set_enabled(false);
                                    }
                                }
                            },
                            Some(parent_folder) => match &self.old_folder_name {
                                None => {
                                    if parent_folder
                                        .borrow()
                                        .folders
                                        .contains_key(self.folder.borrow().name.as_str())
                                    {
                                        ui.set_enabled(false);
                                    }
                                }
                                Some(old_name) => {
                                    if old_name != self.folder.borrow().name.as_str()
                                        && parent_folder
                                            .borrow()
                                            .folders
                                            .contains_key(self.folder.borrow().name.as_str())
                                    {
                                        ui.set_enabled(false);
                                    }
                                }
                            },
                        }

                        if ui.button("Save").clicked() {
                            self.new_collection_windows_open = false;
                            match &self.old_collection_name {
                                None => {}
                                Some(old_name) => {
                                    app_data.collections.remove(old_name.clone());
                                }
                            }
                            match &self.parent_folder {
                                None => {}
                                Some(parent_folder) => {
                                    match &self.old_folder_name {
                                        None => {}
                                        Some(old_name) => {
                                            parent_folder.borrow_mut().folders.remove(old_name);
                                        }
                                    }
                                    parent_folder.borrow_mut().folders.insert(
                                        self.folder.borrow().name.clone(),
                                        self.folder.clone(),
                                    );
                                }
                            }
                            app_data
                                .collections
                                .insert_or_update(self.new_collection.clone());
                        }
                        ui.set_enabled(true);
                    } else {
                        ui.set_enabled(false);
                        ui.button("Save");
                        ui.set_enabled(true);
                    }
                });
            });
    }
}

impl DataView for NewCollectionWindows {
    type CursorType = i32;

    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        app_data: &mut AppData,
        cursor: Self::CursorType,
    ) {
        app_data.lock_ui(
            "new_collection".to_string(),
            self.new_collection_windows_open,
        );
        let mut new_collection_windows_open = self.new_collection_windows_open;
        egui::Window::new(self.title_name.clone())
            .default_open(true)
            .max_width(500.0)
            .max_width(500.0)
            .max_height(600.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut new_collection_windows_open)
            .show(ui.ctx(), |ui| {
                ui.label("Name");
                utils::text_edit_singleline(ui, &mut self.folder.borrow_mut().name);
                ui.horizontal(|ui| {
                    for x in NewCollectionContentType::iter() {
                        if x == NewCollectionContentType::Variables && self.parent_folder.is_some()
                        {
                            continue;
                        }
                        ui.selectable_value(
                            &mut self.new_collection_content_type,
                            x.clone(),
                            x.to_string(),
                        );
                    }
                });
                ui.add_space(VERTICAL_GAP);
                match &self.new_collection_content_type {
                    NewCollectionContentType::Description => {
                        self.build_desc(ui);
                    }
                    NewCollectionContentType::Authorization => {
                        self.build_auth(ui);
                    }
                    NewCollectionContentType::Variables => {
                        self.build_variables(ui);
                    }
                }
                self.bottom_panel(app_data, ui);
            });
        self.new_collection_windows_open = new_collection_windows_open;
    }
}
