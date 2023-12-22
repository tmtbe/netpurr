use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use eframe::emath::pos2;
use egui::{CollapsingHeader, Response, RichText, Ui};

use crate::data::{AppData, CentralRequestItem, Collection, CollectionFolder, HttpRecord};
use crate::panels::new_collection_windows::NewCollectionWindows;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct CollectionsPanel {
    new_collection_windows: NewCollectionWindows,
}

impl DataView for CollectionsPanel {
    type CursorType = i32;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        if ui.link("+ New Collection").clicked() {
            self.new_collection_windows.open_collection(None);
        };
        self.render_collection_item(ui, app_data);
        self.new_collection_windows.set_and_render(app_data, 0, ui);
    }
}

impl CollectionsPanel {
    fn set_folder(
        &mut self,
        ui: &mut Ui,
        app_data: &mut AppData,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
        path: String,
    ) {
        let folder_name = folder.borrow().name.clone();
        let response = CollapsingHeader::new(folder_name.clone())
            .default_open(false)
            .show(ui, |ui| {
                let folders = folder.borrow().folders.clone();
                for (name, cf) in folders.iter() {
                    self.set_folder(
                        ui,
                        app_data,
                        collection.clone(),
                        parent_folder.clone(),
                        cf.clone(),
                        format!("{}/{}", path, name),
                    )
                }
                let requests = folder.borrow().requests.clone();
                self.render_request(
                    ui,
                    app_data,
                    collection.folder.borrow().name.clone(),
                    &path,
                    requests,
                );
            })
            .header_response;

        self.popup_folder_item(
            ui,
            app_data,
            collection,
            parent_folder,
            folder,
            path,
            folder_name,
            &response,
        );
    }

    fn popup_folder_item(
        &mut self,
        ui: &mut Ui,
        app_data: &mut AppData,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
        path: String,
        folder_name: String,
        response: &Response,
    ) {
        let mut remove_id = None;
        let popup_id = ui.make_persistent_id(
            "folder_item_popup_menu_".to_string()
                + format!("{}/{}", path, folder.borrow().name.as_str()).as_str(),
        );
        if response.secondary_clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        utils::popup_widget(
            ui,
            popup_id,
            &response,
            pos2(response.rect.right(), response.rect.bottom()),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_width(100.0)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if utils::select_label(ui, "Edit").clicked() {
                                self.new_collection_windows.open_folder(
                                    collection.clone(),
                                    parent_folder.clone(),
                                    Some(folder.clone()),
                                );
                            }
                            if utils::select_label(ui, "Add Folder").clicked() {
                                self.new_collection_windows.open_folder(
                                    collection.clone(),
                                    folder.clone(),
                                    None,
                                );
                            }
                            if utils::select_label(ui, "Duplicate").clicked() {
                                let new_name = utils::build_copy_name(
                                    folder_name.clone(),
                                    parent_folder
                                        .borrow()
                                        .folders
                                        .iter()
                                        .map(|(k, _)| k.clone())
                                        .collect(),
                                );
                                let new_folder = Rc::new(RefCell::new(CollectionFolder {
                                    name: new_name.clone(),
                                    desc: folder.borrow().desc.clone(),
                                    auth: folder.borrow().auth.clone(),
                                    is_root: folder.borrow().is_root,
                                    requests: folder.borrow().requests.clone(),
                                    folders: folder.borrow().folders.clone(),
                                }));
                                parent_folder
                                    .borrow_mut()
                                    .folders
                                    .insert(new_name.clone(), new_folder);
                                app_data
                                    .collections
                                    .update(collection.folder.borrow().name.clone());
                            }
                            if utils::select_label(ui, "Remove").clicked() {
                                remove_id = Some(folder_name.clone());
                            }
                        });
                    });
            },
        );
        remove_id.map(|id| {
            parent_folder.borrow_mut().folders.remove(id.as_str());
        });
    }

    fn popup_collection_item(
        &mut self,
        ui: &mut Ui,
        app_data: &mut AppData,
        response: &Response,
        collection_name: &String,
        collection: &Collection,
    ) {
        let popup_id =
            ui.make_persistent_id("collection_item_popup_menu_".to_string() + collection_name);
        if response.secondary_clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        utils::popup_widget(
            ui,
            popup_id,
            &response,
            pos2(response.rect.right(), response.rect.bottom()),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_width(100.0)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if utils::select_label(ui, "Edit").clicked() {
                                self.new_collection_windows
                                    .open_collection(Some(collection.clone()));
                            }
                            if utils::select_label(ui, "Add Folder").clicked() {
                                self.new_collection_windows.open_folder(
                                    collection.clone(),
                                    collection.folder.clone(),
                                    None,
                                );
                            }
                            if utils::select_label(ui, "Duplicate").clicked() {
                                let new_collections = collection.clone();
                                let new_name = utils::build_copy_name(
                                    collection_name.to_string(),
                                    app_data
                                        .collections
                                        .get_data()
                                        .iter()
                                        .map(|(k, _)| k.to_string())
                                        .collect(),
                                );
                                new_collections.folder.borrow_mut().name = new_name;
                                app_data.collections.insert_or_update(new_collections);
                            }
                            if utils::select_label(ui, "Remove").clicked() {
                                app_data
                                    .collections
                                    .remove(collection.folder.borrow().name.clone());
                            }
                        });
                    });
            },
        );
    }

    fn render_collection_item(&mut self, ui: &mut Ui, app_data: &mut AppData) {
        fn circle_icon(ui: &mut Ui, openness: f32, response: &Response) {
            let stroke = ui.style().interact(&response).fg_stroke;
            let radius = egui::lerp(2.0..=3.0, openness);
            ui.painter()
                .circle_filled(response.rect.center(), radius, stroke.color);
        }
        for (collection_name, collection) in app_data.collections.get_data().iter() {
            let response = CollapsingHeader::new(RichText::new(collection_name).strong())
                .icon(circle_icon)
                .default_open(false)
                .show(ui, |ui| {
                    let folders = collection.folder.borrow().folders.clone();
                    for (cf_name, cf) in folders.iter() {
                        self.set_folder(
                            ui,
                            app_data,
                            collection.clone(),
                            collection.folder.clone(),
                            cf.clone(),
                            format!("{}/{}", collection_name, cf_name.clone()),
                        );
                    }
                    let requests = collection.folder.borrow().requests.clone();
                    self.render_request(
                        ui,
                        app_data,
                        collection_name.to_string(),
                        collection_name,
                        requests,
                    );
                })
                .header_response;
            self.popup_collection_item(ui, app_data, &response, collection_name, collection);
        }
    }

    fn popup_request_item(
        &mut self,
        ui: &mut Ui,
        app_data: &mut AppData,
        collection_name: String,
        response: &Response,
        request: &HttpRecord,
        path: &String,
    ) {
        let popup_id = ui.make_persistent_id(
            "request_item_popup_menu_".to_string() + path + "_" + request.name.as_str(),
        );
        if response.secondary_clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        utils::popup_widget(
            ui,
            popup_id,
            &response,
            pos2(response.rect.right(), response.rect.bottom()),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_width(100.0)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if utils::select_label(ui, "Open in New Table").clicked() {
                                let mut id = path.clone() + "/" + request.name.as_str();
                                if app_data
                                    .central_request_data_list
                                    .data_map
                                    .contains_key(id.as_str())
                                {
                                    id = utils::build_copy_name(
                                        id.clone(),
                                        app_data
                                            .central_request_data_list
                                            .data_map
                                            .iter()
                                            .map(|(k, _)| k.clone())
                                            .collect(),
                                    );
                                }
                                app_data
                                    .central_request_data_list
                                    .add_crt(CentralRequestItem {
                                        id,
                                        collection_path: Some(path.clone()),
                                        rest: request.clone(),
                                    })
                            }
                            if utils::select_label(ui, "Duplicate").clicked() {
                                let (_, folder) =
                                    app_data.collections.get_mut_folder_with_path(path.clone());
                                folder.map(|f| {
                                    let cf = f.borrow().clone();
                                    let request = cf.requests.get(request.name.as_str());
                                    request.map(|r| {
                                        let mut new_request = r.clone();
                                        let name = new_request.name.clone();
                                        let new_name = utils::build_copy_name(
                                            name,
                                            f.borrow()
                                                .requests
                                                .iter()
                                                .map(|(k, v)| k.clone())
                                                .collect(),
                                        );
                                        new_request.name = new_name.to_string();
                                        f.borrow_mut()
                                            .requests
                                            .insert(new_name.to_string(), new_request);
                                        app_data.collections.update(collection_name.to_string());
                                    });
                                });
                            }
                            if utils::select_label(ui, "Remove").clicked() {
                                let (_, folder) =
                                    app_data.collections.get_mut_folder_with_path(path.clone());
                                folder.map(|f| {
                                    f.borrow_mut().requests.remove(request.name.as_str());
                                });
                            }
                        });
                    });
            },
        );
    }

    fn render_request(
        &mut self,
        ui: &mut Ui,
        app_data: &mut AppData,
        collection_name: String,
        path: &String,
        requests: BTreeMap<String, HttpRecord>,
    ) {
        for (_, hr) in requests.iter() {
            let lb = utils::build_rest_ui_header(hr.clone(), ui);
            let button = ui.button(lb);
            if button.clicked() {
                app_data
                    .central_request_data_list
                    .add_crt(CentralRequestItem {
                        id: path.clone() + "/" + hr.name.as_str(),
                        collection_path: Some(path.clone()),
                        rest: hr.clone(),
                    })
            }
            self.popup_request_item(ui, app_data, collection_name.clone(), &button, &hr, path)
        }
    }
}
