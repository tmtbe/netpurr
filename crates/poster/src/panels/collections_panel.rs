use std::cell::RefCell;
use std::rc::Rc;

use egui::{CollapsingHeader, Ui};

use crate::data::{AppData, CentralRequestItem, CollectionFolder};
use crate::panels::DataView;
use crate::utils;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct CollectionsPanel {}

impl DataView for CollectionsPanel {
    type CursorType = i32;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        ui.link("+ New Collection");
        for (c_name, collection) in app_data.collections.get_data().iter() {
            CollapsingHeader::new(c_name)
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
                });
        }
    }
}

impl CollectionsPanel {
    fn set_folder(
        &self,
        app_data: &mut AppData,
        ui: &mut Ui,
        cf: Rc<RefCell<CollectionFolder>>,
        path: String,
    ) {
        CollapsingHeader::new(cf.borrow().name.clone())
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
            });
    }
}
