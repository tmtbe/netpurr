use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use egui::{CollapsingHeader, Response, RichText, Ui};

use crate::data::central_request_data::CentralRequestItem;
use crate::data::collections::{Collection, CollectionFolder};
use crate::data::export::{Export, ExportType};
use crate::data::http::HttpRecord;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::utils;
use crate::windows::new_collection_windows::NewCollectionWindows;
use crate::windows::save_windows::SaveWindows;

#[derive(Default)]
pub struct CollectionsPanel {}

impl CollectionsPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        if ui.link("+ New Collection").clicked() {
            operation.add_window(Box::new(
                NewCollectionWindows::default().with_open_collection(None),
            ));
        };
        self.render_collection_item(ui, operation, workspace_data);
    }
    fn set_folder(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
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
                        operation,
                        workspace_data,
                        collection.clone(),
                        parent_folder.clone(),
                        cf.clone(),
                        format!("{}/{}", path, name),
                    )
                }
                let requests = folder.borrow().requests.clone();
                self.render_request(
                    ui,
                    operation,
                    workspace_data,
                    collection.folder.borrow().name.clone(),
                    &path,
                    requests,
                );
            })
            .header_response;

        self.popup_folder_item(
            operation,
            workspace_data,
            collection,
            parent_folder,
            folder,
            folder_name,
            response,
        );
    }

    fn popup_folder_item(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
        folder_name: String,
        response: Response,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Edit").clicked() {
                operation.add_window(Box::new(NewCollectionWindows::default().with_open_folder(
                    collection.clone(),
                    parent_folder.clone(),
                    Some(folder.clone()),
                )));
                ui.close_menu();
            }
            if utils::select_label(ui, "Add Folder").clicked() {
                operation.add_window(Box::new(NewCollectionWindows::default().with_open_folder(
                    collection.clone(),
                    folder.clone(),
                    None,
                )));
                ui.close_menu();
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
                    parent_path: folder.borrow().parent_path.clone(),
                    desc: folder.borrow().desc.clone(),
                    auth: folder.borrow().auth.clone(),
                    is_root: folder.borrow().is_root,
                    requests: folder.borrow().requests.clone(),
                    folders: folder.borrow().folders.clone(),
                    pre_request_script: folder.borrow().pre_request_script.clone(),
                    test_script: folder.borrow().test_script.clone(),
                }));
                workspace_data.collection_insert_folder(parent_folder.clone(), new_folder.clone());
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                workspace_data.remove_folder(parent_folder, folder_name.clone());
                ui.close_menu();
            }
        });
    }

    fn popup_collection_item(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        response: Response,
        collection_name: &String,
        collection: &Collection,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Edit").clicked() {
                operation.add_window(Box::new(
                    NewCollectionWindows::default().with_open_collection(Some(collection.clone())),
                ));
                ui.close_menu();
            }
            if utils::select_label(ui, "Add Folder").clicked() {
                operation.add_window(Box::new(NewCollectionWindows::default().with_open_folder(
                    collection.clone(),
                    collection.folder.clone(),
                    None,
                )));
                ui.close_menu();
            }
            if utils::select_label(ui, "Duplicate").clicked() {
                let new_collections = collection.clone();
                let new_name = utils::build_copy_name(
                    collection_name.to_string(),
                    workspace_data.get_collection_names(),
                );
                new_collections.folder.borrow_mut().name = new_name;
                workspace_data.add_collection(new_collections);
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                workspace_data.remove_collection(collection.folder.borrow().name.clone());
                ui.close_menu();
            }
            ui.separator();
            if utils::select_label(ui, "Export").clicked() {
                ui.close_menu();
                let export = Export {
                    export_type: ExportType::Collection,
                    collection: Some(collection.clone()),
                };
                if let Ok(json) = serde_json::to_string(&export) {
                    let file_name = format!("collection-{}.json", collection.folder.borrow().name);
                    if let Some(path) = rfd::FileDialog::new().set_file_name(file_name).save_file()
                    {
                        match File::create(path) {
                            Ok(mut file) => match file.write_all(json.as_bytes()) {
                                Ok(_) => {
                                    operation.add_success_toast("Export collection success.");
                                }
                                Err(e) => {
                                    operation.add_error_toast(format!(
                                        "Export collection file failed: {}",
                                        e.to_string()
                                    ));
                                }
                            },
                            Err(e) => {
                                operation.add_error_toast(format!(
                                    "Export collection file failed: {}",
                                    e.to_string()
                                ));
                            }
                        }
                    }
                }
            }
        });
    }

    fn render_collection_item(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        fn circle_icon(ui: &mut Ui, openness: f32, response: &Response) {
            let stroke = ui.style().interact(&response).fg_stroke;
            let radius = egui::lerp(2.0..=3.0, openness);
            ui.painter()
                .circle_filled(response.rect.center(), radius, stroke.color);
        }
        for (collection_name, collection) in workspace_data.get_collections().iter() {
            let response = CollapsingHeader::new(RichText::new(collection_name).strong())
                .icon(circle_icon)
                .default_open(false)
                .show(ui, |ui| {
                    let folders = collection.folder.borrow().folders.clone();
                    for (cf_name, cf) in folders.iter() {
                        self.set_folder(
                            ui,
                            operation,
                            workspace_data,
                            collection.clone(),
                            collection.folder.clone(),
                            cf.clone(),
                            format!("{}/{}", collection_name, cf_name.clone()),
                        );
                    }
                    let requests = collection.folder.borrow().requests.clone();
                    self.render_request(
                        ui,
                        operation,
                        workspace_data,
                        collection_name.to_string(),
                        collection_name,
                        requests,
                    );
                })
                .header_response;
            self.popup_collection_item(
                operation,
                workspace_data,
                response,
                collection_name,
                collection,
            );
        }
    }

    fn popup_request_item(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
        response: Response,
        request: &HttpRecord,
        path: &String,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Open in New Table").clicked() {
                let mut crt_id = path.clone() + "/" + request.name.as_str();
                if workspace_data.contains_crt_id(crt_id.clone()) {
                    crt_id =
                        utils::build_copy_name(crt_id.clone(), workspace_data.get_crt_id_set());
                }
                workspace_data.add_crt(CentralRequestItem {
                    id: crt_id,
                    collection_path: Some(path.clone()),
                    rest: request.clone(),
                    ..Default::default()
                });
                ui.close_menu();
            }
            if utils::select_label(ui, "Save as").clicked() {
                operation.add_window(Box::new(SaveWindows::default().with(
                    request.clone(),
                    Some(path.clone()),
                    false,
                )));
                ui.close_menu();
            }
            if utils::select_label(ui, "Edit").clicked() {
                operation.add_window(Box::new(SaveWindows::default().with(
                    request.clone(),
                    Some(path.clone()),
                    true,
                )));
                ui.close_menu();
            }
            if utils::select_label(ui, "Duplicate").clicked() {
                let (_, folder) = workspace_data.get_folder_with_path(path.clone());
                folder.map(|f| {
                    let cf = f.borrow().clone();
                    let request = cf.requests.get(request.name.as_str());
                    request.map(|r| {
                        let mut new_request = r.clone();
                        let name = new_request.name.clone();
                        let new_name = utils::build_copy_name(
                            name,
                            f.borrow().requests.iter().map(|(k, v)| k.clone()).collect(),
                        );
                        new_request.name = new_name.to_string();
                        workspace_data.collection_insert_http_record(f.clone(), new_request);
                    });
                });
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                let (_, folder) = workspace_data.get_folder_with_path(path.clone());
                folder.map(|f| {
                    workspace_data.collection_remove_http_record(f.clone(), request.name.clone());
                });
                ui.close_menu();
            }
        });
    }

    fn render_request(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
        path: &String,
        requests: BTreeMap<String, HttpRecord>,
    ) {
        for (_, hr) in requests.iter() {
            let lb = utils::build_rest_ui_header(hr.clone(), None, ui);
            let button = ui.button(lb);
            if button.clicked() {
                workspace_data.add_crt(CentralRequestItem {
                    id: path.clone() + "/" + hr.name.as_str(),
                    collection_path: Some(path.clone()),
                    rest: hr.clone(),
                    ..Default::default()
                })
            }
            self.popup_request_item(
                operation,
                workspace_data,
                collection_name.clone(),
                button,
                &hr,
                path,
            )
        }
    }
}
