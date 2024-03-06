use std::cell::RefCell;
use std::collections::BTreeMap;
use std::default::Default;
use std::rc::Rc;

use egui::{Align, Button, Checkbox, Layout, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use netpurr_core::data::auth::{Auth, AuthType};
use netpurr_core::data::collections::{Collection, CollectionFolder};
use netpurr_core::data::environment::{
    EnvironmentItem, EnvironmentItemValue, EnvironmentValueType,
};
use netpurr_core::data::http::Request;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::auth_panel::AuthPanel;
use crate::panels::openapi_show_request_panel::OpenApiShowRequestPanel;
use crate::panels::request_pre_script_panel::RequestPreScriptPanel;
use crate::panels::test_script_panel::TestScriptPanel;
use crate::panels::VERTICAL_GAP;
use crate::utils;
use crate::utils::HighlightValue;

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
    request_pre_script_panel: RequestPreScriptPanel,
    test_script_panel: TestScriptPanel,
    search_input: String,
    openapi_panel: OpenApiShowRequestPanel,
}

#[derive(Clone, EnumString, EnumIter, PartialEq, Display)]
enum NewCollectionContentType {
    Description,
    Authorization,
    Variables,
    PreRequestScript,
    Tests,
    OpenAPI,
}

impl Default for NewCollectionContentType {
    fn default() -> Self {
        NewCollectionContentType::Description
    }
}

impl Window for NewCollectionWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new(self.title_name.clone())
            .modal(true)
            .max_width(800.0)
            .min_width(500.0)
            .max_height(600.0)
            .collapsible(false)
            .resizable(true)
    }

    fn set_open(&mut self, open: bool) {
        self.new_collection_windows_open = open;
    }

    fn get_open(&self) -> bool {
        self.new_collection_windows_open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        ui.label("Name");
        utils::text_edit_singleline_filter_justify(ui, &mut self.folder.borrow_mut().name);
        let parent_auth =
            workspace_data.get_path_parent_auth(self.folder.borrow().parent_path.clone());
        ui.horizontal(|ui| {
            for x in NewCollectionContentType::iter() {
                if x == NewCollectionContentType::Variables && self.parent_folder.is_some() {
                    continue;
                }
                if x == NewCollectionContentType::OpenAPI && self.parent_folder.is_some() {
                    continue;
                }
                ui.selectable_value(
                    &mut self.new_collection_content_type,
                    x.clone(),
                    utils::build_with_count_ui_header(
                        x.to_string(),
                        NewCollectionWindows::get_count(
                            self.folder.clone(),
                            x,
                            &parent_auth,
                            self.new_collection.envs.items.len(),
                        ),
                        ui,
                    ),
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
            NewCollectionContentType::PreRequestScript => {
                let script = self.folder.borrow().pre_request_script.clone();
                let mut env = BTreeMap::default();
                for et in self.new_collection.envs.items.iter() {
                    env.insert(
                        et.key.clone(),
                        EnvironmentItemValue {
                            value: et.value.clone(),
                            scope: self.folder.borrow().name.clone(),
                            value_type: EnvironmentValueType::String,
                        },
                    );
                }
                let path = self.folder.borrow().parent_path.clone();
                let name = self.folder.borrow().name.clone();
                self.folder.borrow_mut().pre_request_script =
                    self.request_pre_script_panel.set_and_render(
                        ui,
                        &operation,
                        format!("{}/{}", path, name),
                        script,
                        workspace_data.get_path_parent_scripts(path).0,
                        Request::default(),
                        env,
                        "collection".to_string(),
                    );
            }
            NewCollectionContentType::Tests => {
                let script = self.folder.borrow().test_script.clone();
                self.folder.borrow_mut().test_script =
                    self.test_script_panel
                        .set_and_render(ui, script, "collection".to_string())
            }
            NewCollectionContentType::OpenAPI => {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        self.openapi_panel.render(
                            ui,
                            workspace_data,
                            &operation,
                            self.new_collection.clone(),
                        );
                    });
            }
        }
        self.bottom_panel(workspace_data, ui);
    }
}
impl NewCollectionWindows {
    fn get_count(
        cf: Rc<RefCell<CollectionFolder>>,
        panel_enum: NewCollectionContentType,
        parent_auth: &Auth,
        vars: usize,
    ) -> HighlightValue {
        match panel_enum {
            NewCollectionContentType::Description => {
                if cf.borrow().desc.is_empty() {
                    HighlightValue::None
                } else {
                    HighlightValue::Has
                }
            }
            NewCollectionContentType::Authorization => {
                match cf.borrow().auth.get_final_type(parent_auth.clone()) {
                    AuthType::InheritAuthFromParent => HighlightValue::None,
                    AuthType::NoAuth => HighlightValue::None,
                    AuthType::BearerToken => HighlightValue::Has,
                    AuthType::BasicAuth => HighlightValue::Has,
                }
            }
            NewCollectionContentType::Variables => HighlightValue::Usize(vars),
            NewCollectionContentType::PreRequestScript => {
                if !cf.borrow().pre_request_script.is_empty() {
                    HighlightValue::Has
                } else {
                    HighlightValue::None
                }
            }
            NewCollectionContentType::Tests => {
                if !cf.borrow().test_script.is_empty() {
                    HighlightValue::Has
                } else {
                    HighlightValue::None
                }
            }
            NewCollectionContentType::OpenAPI => HighlightValue::None,
        }
    }
    pub fn with_open_collection(mut self, collection: Option<Collection>) -> Self {
        self.new_collection_windows_open = true;
        match collection {
            None => {
                self.new_collection = Collection::default();
                self.new_collection.folder.borrow_mut().is_root = true;
                self.new_collection.folder.borrow_mut().parent_path = ".".to_string();
                self.title_name = "CREATE A NEW COLLECTION".to_string();
            }
            Some(collection) => {
                self.new_collection = collection.duplicate(collection.folder.borrow().name.clone());
                self.old_collection_name = Some(self.new_collection.folder.borrow().name.clone());
                self.title_name = "EDIT COLLECTION".to_string();
            }
        }

        self.new_collection_content_type = NewCollectionContentType::Description;
        self.folder = self.new_collection.folder.clone();
        self.parent_folder = None;
        self
    }
    pub fn with_open_folder(
        mut self,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Option<Rc<RefCell<CollectionFolder>>>,
    ) -> Self {
        self.new_collection_windows_open = true;
        match folder {
            None => {
                self.new_collection = collection;
                self.title_name = "CREATE A NEW FOLDER".to_string();
                self.folder = Rc::new(RefCell::new(CollectionFolder::default()));
            }
            Some(cf) => {
                self.new_collection = collection;
                self.old_folder_name = Some(cf.borrow().name.clone());
                self.title_name = "EDIT FOLDER".to_string();
                self.folder = Rc::new(RefCell::new(
                    cf.borrow().duplicate(cf.borrow().name.clone()),
                ));
            }
        }
        self.parent_folder = Some(parent_folder.clone());
        self.new_collection_content_type = NewCollectionContentType::Description;
        self
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
        utils::text_edit_multiline_justify(ui, &mut self.folder.borrow_mut().desc);
        ui.add_space(VERTICAL_GAP);
    }

    fn bottom_panel(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("new_collection_bottom_panel")
            .resizable(false)
            .min_height(0.0)
            .show_inside(ui, |ui| {
                ui.add_space(VERTICAL_GAP);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Cancel").clicked() {
                        self.new_collection_windows_open = false;
                    }
                    let name = self.new_collection.folder.borrow().name.clone();
                    if name != "" {
                        match &self.parent_folder {
                            None => match &self.old_collection_name {
                                None => {
                                    if workspace_data
                                        .get_collections()
                                        .contains_key(self.folder.borrow().name.as_str())
                                    {
                                        ui.set_enabled(false)
                                    }
                                }
                                Some(old_name) => {
                                    if old_name != self.folder.borrow().name.as_str()
                                        && workspace_data
                                            .get_collections()
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
                            let parent_folder = self.parent_folder.clone();
                            match parent_folder {
                                None => {
                                    // means save collection
                                    self.save_collection(workspace_data);
                                }
                                Some(parent_folder) => {
                                    // means save folder
                                    self.save_folder(workspace_data, parent_folder);
                                }
                            }
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

    fn save_folder(
        &mut self,
        workspace_data: &mut WorkspaceData,
        parent_folder: Rc<RefCell<CollectionFolder>>,
    ) {
        match &self.old_folder_name {
            None => {
                // means add new folder
                workspace_data.add_folder(parent_folder.clone(), self.folder.clone());
            }
            Some(old_name) => {
                // means edit old folder
                workspace_data.update_folder_info(
                    old_name.clone(),
                    parent_folder.clone(),
                    self.folder.clone(),
                );
            }
        }
    }

    fn save_collection(&mut self, workspace_data: &mut WorkspaceData) {
        match &self.old_collection_name {
            None => {
                // means add new collection
                workspace_data.add_collection(self.new_collection.clone());
            }
            Some(old_name) => {
                // means edit old collection
                workspace_data
                    .update_collection_info(old_name.clone(), self.new_collection.clone());
            }
        }
    }
}
