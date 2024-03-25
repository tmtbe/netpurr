use std::fs::File;
use std::io::Write;

use egui::{Direction, Label, Layout, RichText, Sense, Ui, Vec2, Widget};

use crate::data::config_data::ConfigData;
use crate::data::export::{Export, ExportType};
use crate::operation::operation::Operation;
use crate::panels::{HORIZONTAL_GAP, VERTICAL_GAP};
use crate::utils;
use crate::widgets::empty_container::EmptyContainer;
use crate::windows::new_collection_windows::NewCollectionWindows;
use netpurr_core::data::workspace_data::WorkspaceData;

pub struct MatrixLabel {
    matrix_label_type: MatrixLabelType,
}

pub enum MatrixLabelType {
    Collection(String),
    Add,
}

impl MatrixLabel {
    pub fn new(matrix_label_type: MatrixLabelType) -> Self {
        MatrixLabel { matrix_label_type }
    }
    pub fn render(
        self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        config_data: &mut ConfigData,
        operation: &Operation,
    ) {
        let size = Vec2 { x: 150.0, y: 150.0 };
        EmptyContainer::default()
            .with_stroke(true)
            .default_size(size)
            .show(ui, |ui| {
                let max = ui.max_rect();
                let response = ui.allocate_rect(max, Sense::click());
                let mut content_ui = ui.child_ui(max, *ui.layout());
                match &self.matrix_label_type {
                    MatrixLabelType::Collection(collection_name) => {
                        content_ui.vertical(|ui| {
                            ui.add_space(VERTICAL_GAP);
                            ui.horizontal(|ui| {
                                ui.add_space(HORIZONTAL_GAP);
                                Label::new(RichText::from("Collection").strong())
                                    .selectable(false)
                                    .ui(ui);
                                utils::add_right_space(ui, 25.0);
                                ui.menu_button("...", |ui| {
                                    self.more_button(
                                        workspace_data,
                                        operation,
                                        collection_name.clone(),
                                        ui,
                                    );
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(HORIZONTAL_GAP);
                                Label::new(RichText::from(collection_name.clone()).heading())
                                    .selectable(false)
                                    .ui(ui);
                            });
                        });
                        if response.clicked() {
                            config_data.set_select_collection(Some(collection_name.clone()));
                        }
                    }
                    MatrixLabelType::Add => {
                        content_ui.with_layout(
                            Layout::centered_and_justified(Direction::TopDown),
                            |ui| {
                                Label::new(RichText::from("+").heading())
                                    .selectable(false)
                                    .ui(ui);
                            },
                        );
                        if response.clicked() {
                            operation.add_window(Box::new(
                                NewCollectionWindows::default().with_open_collection(None),
                            ));
                        }
                    }
                }
            })
    }

    fn more_button(
        &self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        collection_name: String,
        ui: &mut Ui,
    ) {
        workspace_data
            .get_collection_by_name(collection_name.clone())
            .map(|collection| {
                if utils::select_label(ui, "Edit").clicked() {
                    operation.add_window(Box::new(
                        NewCollectionWindows::default()
                            .with_open_collection(Some(collection.clone())),
                    ));
                    ui.close_menu();
                }
                if utils::select_label(ui, "Duplicate").clicked() {
                    let new_name = utils::build_copy_name(
                        collection_name.clone(),
                        workspace_data.get_collection_names(),
                    );
                    let new_collections = collection.duplicate(new_name);
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
                        openapi: None,
                        info: None,
                        export_type: ExportType::Collection,
                        collection: Some(collection.clone()),
                    };
                    if let Ok(json) = serde_json::to_string(&export) {
                        let file_name =
                            format!("collection-{}.json", collection.folder.borrow().name);
                        if let Some(path) =
                            rfd::FileDialog::new().set_file_name(file_name).save_file()
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
}
