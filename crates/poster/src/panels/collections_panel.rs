use std::cell::RefCell;
use std::rc::Rc;

use eframe::emath::pos2;
use egui::{CollapsingHeader, RichText, Ui};

use crate::data::{AppData, CentralRequestItem, CollectionFolder};
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
        fn circle_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
            let stroke = ui.style().interact(&response).fg_stroke;
            let radius = egui::lerp(2.0..=3.0, openness);
            ui.painter()
                .circle_filled(response.rect.center(), radius, stroke.color);
        }
        if ui.link("+ New Collection").clicked() {
            self.new_collection_windows.open_collection(None);
        };
        for (c_name, collection) in app_data.collections.get_data().iter() {
            let response = CollapsingHeader::new(RichText::new(c_name).strong())
                .icon(circle_icon)
                .default_open(false)
                .show(ui, |ui| {
                    for (cf_name, cf) in collection.folder.borrow().folders.iter() {
                        self.set_folder(
                            app_data,
                            ui,
                            cf.clone(),
                            format!("{}/{}", c_name, cf_name.clone()),
                        );
                    }
                    for (_, hr) in collection.folder.borrow().requests.iter() {
                        let lb = utils::build_rest_ui_header(hr.clone(), ui);
                        let button = ui.button(lb);
                        if button.clicked() {
                            app_data
                                .central_request_data_list
                                .add_crt(CentralRequestItem {
                                    id: hr.name.clone(),
                                    collection_path: Some(c_name.clone()),
                                    rest: hr.clone(),
                                })
                        }
                    }
                })
                .header_response;
            let popup_id =
                ui.make_persistent_id("collection_item_popup_menu_".to_string() + c_name);
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
        self.new_collection_windows.set_and_render(app_data, 0, ui);
    }
}

impl CollectionsPanel {
    fn set_folder(
        &mut self,
        app_data: &mut AppData,
        ui: &mut Ui,
        cf: Rc<RefCell<CollectionFolder>>,
        path: String,
    ) {
        let response = CollapsingHeader::new(cf.borrow().name.as_str())
            .default_open(false)
            .show(ui, |ui| {
                for (name, cf) in cf.borrow().folders.iter() {
                    self.set_folder(app_data, ui, cf.clone(), format!("{}/{}", path, name))
                }
                for (_, hr) in cf.borrow().requests.iter() {
                    let lb = utils::build_rest_ui_header(hr.clone(), ui);
                    let button = ui.button(lb);
                    if button.clicked() {
                        app_data
                            .central_request_data_list
                            .add_crt(CentralRequestItem {
                                id: hr.name.clone(),
                                collection_path: Some(path.clone()),
                                rest: hr.clone(),
                            })
                    }
                }
            })
            .header_response;

        let popup_id = ui.make_persistent_id(
            "collection_item_popup_menu_".to_string()
                + format!("{}/{}", path, cf.borrow().name.as_str()).as_str(),
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
                                //self.new_collection_windows.open_collection(Some(collection.clone()));
                            }
                            if utils::select_label(ui, "Remove").clicked() {}
                        });
                    });
            },
        );
    }
}
