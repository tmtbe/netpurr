use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use egui::{CollapsingHeader, Response, RichText, Ui};
use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::data::{
    CentralRequestItem, Collection, CollectionFolder, Export, ExportType, HttpRecord, WorkspaceData,
};
use crate::operation::Operation;
use crate::panels::DataView;
use crate::utils;

#[derive(Default)]
pub struct CollectionsPanel {}

impl DataView for CollectionsPanel {
    type CursorType = i32;

    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        if ui.link("+ New Collection").clicked() {
            operation.open_windows().open_collection(None);
        };
        self.render_collection_item(ui, operation, workspace_data);
    }
}

impl CollectionsPanel {
    fn set_folder(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
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
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
        folder_name: String,
        response: Response,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Edit").clicked() {
                operation.open_windows().open_folder(
                    collection.clone(),
                    parent_folder.clone(),
                    Some(folder.clone()),
                );
                ui.close_menu();
            }
            if utils::select_label(ui, "Add Folder").clicked() {
                operation
                    .open_windows()
                    .open_folder(collection.clone(), folder.clone(), None);
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
                }));
                workspace_data
                    .collections
                    .insert_folder(parent_folder.clone(), new_folder.clone());
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                parent_folder
                    .borrow_mut()
                    .folders
                    .remove(folder_name.as_str());
                ui.close_menu();
            }
        });
    }

    fn popup_collection_item(
        &mut self,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        response: Response,
        collection_name: &String,
        collection: &Collection,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Edit").clicked() {
                operation
                    .open_windows()
                    .open_collection(Some(collection.clone()));
                ui.close_menu();
            }
            if utils::select_label(ui, "Add Folder").clicked() {
                operation.open_windows().open_folder(
                    collection.clone(),
                    collection.folder.clone(),
                    None,
                );
                ui.close_menu();
            }
            if utils::select_label(ui, "Duplicate").clicked() {
                let new_collections = collection.clone();
                let new_name = utils::build_copy_name(
                    collection_name.to_string(),
                    workspace_data
                        .collections
                        .get_data()
                        .iter()
                        .map(|(k, _)| k.to_string())
                        .collect(),
                );
                new_collections.folder.borrow_mut().name = new_name;
                workspace_data
                    .collections
                    .insert_collection(new_collections);
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                workspace_data
                    .collections
                    .remove_collection(collection.folder.borrow().name.clone());
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
                                    operation.toasts().add(Toast {
                                        text: format!("Export collection success.").into(),
                                        kind: ToastKind::Info,
                                        options: ToastOptions::default()
                                            .duration_in_seconds(2.0)
                                            .show_progress(true),
                                    });
                                }
                                Err(e) => {
                                    operation.toasts().add(Toast {
                                        text: format!(
                                            "Export collection file failed: {}",
                                            e.to_string()
                                        )
                                        .into(),
                                        kind: ToastKind::Error,
                                        options: ToastOptions::default()
                                            .duration_in_seconds(5.0)
                                            .show_progress(true),
                                    });
                                }
                            },
                            Err(e) => {
                                operation.toasts().add(Toast {
                                    text: format!(
                                        "Export collection file failed: {}",
                                        e.to_string()
                                    )
                                    .into(),
                                    kind: ToastKind::Error,
                                    options: ToastOptions::default()
                                        .duration_in_seconds(5.0)
                                        .show_progress(true),
                                });
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
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
    ) {
        fn circle_icon(ui: &mut Ui, openness: f32, response: &Response) {
            let stroke = ui.style().interact(&response).fg_stroke;
            let radius = egui::lerp(2.0..=3.0, openness);
            ui.painter()
                .circle_filled(response.rect.center(), radius, stroke.color);
        }
        for (collection_name, collection) in workspace_data.collections.get_data().iter() {
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
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
        response: Response,
        request: &HttpRecord,
        path: &String,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Open in New Table").clicked() {
                let mut id = path.clone() + "/" + request.name.as_str();
                if workspace_data
                    .central_request_data_list
                    .data_map
                    .contains_key(id.as_str())
                {
                    id = utils::build_copy_name(
                        id.clone(),
                        workspace_data
                            .central_request_data_list
                            .data_map
                            .iter()
                            .map(|(k, _)| k.clone())
                            .collect(),
                    );
                }
                workspace_data
                    .central_request_data_list
                    .add_crt(CentralRequestItem {
                        id,
                        collection_path: Some(path.clone()),
                        rest: request.clone(),
                        ..Default::default()
                    });
                ui.close_menu();
            }
            if utils::select_label(ui, "Save as").clicked() {
                operation
                    .open_windows()
                    .open_save(request.clone(), Some(path.clone()));
                ui.close_menu();
            }
            if utils::select_label(ui, "Edit").clicked() {
                operation
                    .open_windows()
                    .open_edit(request.clone(), path.clone());
                ui.close_menu();
            }
            if utils::select_label(ui, "Duplicate").clicked() {
                let (_, folder) = workspace_data
                    .collections
                    .get_mut_folder_with_path(path.clone());
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
                        workspace_data
                            .collections
                            .insert_http_record(f.clone(), new_request);
                    });
                });
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                let (_, folder) = workspace_data
                    .collections
                    .get_mut_folder_with_path(path.clone());
                folder.map(|f| {
                    f.borrow_mut().requests.remove(request.name.as_str());
                });
                ui.close_menu();
            }
        });
    }

    fn render_request(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
        path: &String,
        requests: BTreeMap<String, HttpRecord>,
    ) {
        for (_, hr) in requests.iter() {
            let lb = utils::build_rest_ui_header(hr.clone(), None, ui);
            let button = ui.button(lb);
            if button.clicked() {
                workspace_data
                    .central_request_data_list
                    .add_crt(CentralRequestItem {
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
