use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{CollapsingHeader, Response, Ui};
use uuid::Uuid;

use netpurr_core::data::central_request_data::CentralRequestItem;
use netpurr_core::data::collections::{Collection, CollectionFolder};
use netpurr_core::data::http::{HttpRecord, Request};
use netpurr_core::data::record::Record;
use netpurr_core::data::workspace_data::WorkspaceData;

use crate::operation::operation::Operation;
use crate::utils;
use crate::windows::new_collection_windows::NewCollectionWindows;
use crate::windows::save_crt_windows::SaveCRTWindows;
use crate::windows::save_windows::SaveWindows;

#[derive(Default)]
pub struct CollectionPanel {}

impl CollectionPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
    ) {
        workspace_data
            .get_collection_by_name(collection_name.clone())
            .map(|collection| {
                if ui.link("+ New Folder").clicked() {
                    operation.add_window(Box::new(
                        NewCollectionWindows::default().with_open_folder(
                            collection.clone(),
                            collection.folder.clone(),
                            None,
                        ),
                    ));
                };
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
                    collection_name.to_string(),
                    requests,
                );
            });
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
        let response = CollapsingHeader::new(format!("{} {}",egui_phosphor::regular::FOLDER,folder_name))
            .default_open(false)
            .show(ui, |ui| {
                let folders = folder.borrow().folders.clone();
                for (name, cf) in folders.iter() {
                    self.set_folder(
                        ui,
                        operation,
                        workspace_data,
                        collection.clone(),
                        folder.clone(),
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
                    path.clone(),
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
            if utils::select_label(ui, "Add Request").clicked() {
                let crt = CentralRequestItem{
                    id: Uuid::new_v4().to_string(),
                    collection_path: None,
                    record: Record::Rest(HttpRecord{
                        name: "New Request".to_string(),
                        desc: "".to_string(),
                        request: Request{
                            method: Default::default(),
                            schema: Default::default(),
                            raw_url: "http://www.httpbin.org/get".to_string(),
                            base_url: "www.httpbin.org/get".to_string(),
                            path_variables: vec![],
                            params: vec![],
                            headers: vec![],
                            body: Default::default(),
                            auth: Default::default(),
                        },
                        .. Default::default()
                    }),
                    test_result: Default::default(),
                    modify_baseline: "".to_string(),
                };
                workspace_data.add_crt(crt.clone());
                operation.add_window(Box::new(SaveCRTWindows::default().with(
                    crt.id.clone(),
                    Some(folder.borrow().get_path()),
                )));
                workspace_data.set_crt_select_id(Some(crt.id.clone()));
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
                let new_folder = Rc::new(RefCell::new(folder.borrow().duplicate(new_name)));
                workspace_data.collection_insert_folder(parent_folder.clone(), new_folder.clone());
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                workspace_data.remove_folder(parent_folder, folder_name.clone());
                ui.close_menu();
            }
        });
    }

    fn popup_request_item(
        &mut self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
        response: Response,
        record: &Record,
        path: String,
    ) {
        response.context_menu(|ui| {
            if utils::select_label(ui, "Open in New Table").clicked() {
                let mut crt_id = path.clone() + "/" + record.name().as_str();
                if workspace_data.contains_crt_id(crt_id.clone()) {
                    crt_id =
                        utils::build_copy_name(crt_id.clone(), workspace_data.get_crt_id_set());
                }
                workspace_data.add_crt(CentralRequestItem {
                    id: crt_id,
                    collection_path: Some(path.clone()),
                    record: record.clone(),
                    ..Default::default()
                });
                ui.close_menu();
            }
            if utils::select_label(ui, "Save as").clicked() {
                operation.add_window(Box::new(SaveWindows::default().with(
                    record.clone(),
                    Some(path.clone()),
                    false,
                )));
                ui.close_menu();
            }
            if utils::select_label(ui, "Edit").clicked() {
                operation.add_window(Box::new(SaveWindows::default().with(
                    record.clone(),
                    Some(path.clone()),
                    true,
                )));
                ui.close_menu();
            }
            if utils::select_label(ui, "Duplicate").clicked() {
                let (_, folder) = workspace_data.get_folder_with_path(path.clone());
                folder.map(|f| {
                    let cf = f.borrow().clone();
                    let request = cf.requests.get(record.name().as_str());
                    request.map(|r| {
                        let mut new_request = r.clone();
                        let name = new_request.name();
                        let new_name = utils::build_copy_name(
                            name,
                            f.borrow().requests.iter().map(|(k, v)| k.clone()).collect(),
                        );
                        new_request.set_name(new_name.to_string());
                        workspace_data.collection_insert_record(f.clone(), new_request);
                    });
                });
                ui.close_menu();
            }
            if utils::select_label(ui, "Remove").clicked() {
                let (_, folder) = workspace_data.get_folder_with_path(path.clone());
                folder.map(|f| {
                    workspace_data.collection_remove_http_record(f.clone(), record.name());
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
        path: String,
        requests: BTreeMap<String, Record>,
    ) {
        for (_, record) in requests.iter() {
            let lb = utils::build_rest_ui_header(record.clone(), None, ui);
            let button = ui.button(lb);
            if button.clicked() {
                workspace_data.add_crt(CentralRequestItem {
                    id: path.clone() + "/" + record.name().as_str(),
                    collection_path: Some(path.clone()),
                    record: record.clone(),
                    ..Default::default()
                })
            }
            self.popup_request_item(
                operation,
                workspace_data,
                collection_name.clone(),
                button,
                &record,
                path.clone(),
            )
        }
    }
}
