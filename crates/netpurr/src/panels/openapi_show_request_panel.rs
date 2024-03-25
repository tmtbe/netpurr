use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{CollapsingHeader, Response, Ui};
use uuid::Uuid;

use netpurr_core::data::collections::{Collection, CollectionFolder};
use netpurr_core::data::record::Record;

use crate::import::openapi::OpenApi;
use crate::operation::operation::Operation;
use crate::utils;
use crate::utils::openapi_help::OpenApiHelp;
use netpurr_core::data::central_request_data::CentralRequestItem;
use netpurr_core::data::workspace_data::WorkspaceData;

#[derive(Default)]
pub struct OpenApiShowRequestPanel {}

impl OpenApiShowRequestPanel {
    pub fn render(
        &self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        collection: Collection,
    ) {
        if let Some(source_openapi) = collection.openapi.clone() {
            let openapi = OpenApi {
                openapi_help: OpenApiHelp {
                    openapi: source_openapi,
                },
            };
            if let Ok(collection) = openapi.to_collection() {
                let folders = collection.folder.borrow().folders.clone();
                for (cf_name, cf) in folders.iter() {
                    self.set_openapi_folder(
                        ui,
                        operation,
                        workspace_data,
                        collection.clone(),
                        collection.folder.clone(),
                        cf.clone(),
                        format!("{}/{}", collection.folder.borrow().name, cf_name.clone()),
                    );
                }
            }
        }
    }
    fn set_openapi_folder(
        &self,
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
                    self.set_openapi_folder(
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
                self.render_openapi_request(
                    ui,
                    operation,
                    workspace_data,
                    collection.folder.borrow().name.clone(),
                    path.clone(),
                    requests,
                );
            })
            .header_response;
    }

    fn render_openapi_request(
        &self,
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
            button.context_menu(|ui| {
                if ui.button("Add to request").clicked() {
                    workspace_data.add_crt(CentralRequestItem {
                        id: Uuid::new_v4().to_string(),
                        collection_path: None,
                        record: record.clone(),
                        ..Default::default()
                    });
                    ui.close_menu();
                }
            });
            self.popup_openapi_request_item(
                operation,
                workspace_data,
                collection_name.clone(),
                button,
                &record,
                path.clone(),
            )
        }
    }
    fn popup_openapi_request_item(
        &self,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        collection_name: String,
        response: Response,
        record: &Record,
        path: String,
    ) {
    }
}
