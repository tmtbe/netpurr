use egui::Ui;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::{AppData, Method};
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::panels::params_panel::ParamsPanel;

#[derive(Default)]
pub struct EditorPanel {
    open_request_panel_enum: RequestPanelEnum,
    params_panel: ParamsPanel,
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq)]
enum RequestPanelEnum {
    Params,
    Authorization,
    Headers,
    Body,
}

impl Default for RequestPanelEnum {
    fn default() -> Self {
        RequestPanelEnum::Params
    }
}

impl DataView for EditorPanel {
    type CursorType = String;
    fn set_and_render(&mut self,app_data: &mut AppData, cursor: Self::CursorType, ui: &mut egui::Ui){
        {
            let data = app_data.central_request_data_list.data_map.get_mut(cursor.as_str()).unwrap();

            ui.vertical(|ui| {
                ui.label(data.rest.request.url.clone());
                ui.separator();
                ui.add_space(HORIZONTAL_GAP);
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("method")
                        .selected_text(data.rest.request.method.clone().to_string())
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            for x in Method::iter() {
                                ui.selectable_value(&mut data.rest.request.method, x.clone(), x.to_string());
                            }
                        });
                    if ui.button("Send").clicked() {}
                    if ui.button("Save").clicked() {}
                    ui.centered_and_justified(|ui| {
                        ui.text_edit_singleline(&mut data.rest.request.url.clone())
                    });
                });
                ui.separator();
                ui.add_space(HORIZONTAL_GAP);
                ui.horizontal(|ui| {
                    for x in RequestPanelEnum::iter() {
                        ui.selectable_value(&mut self.open_request_panel_enum, x.clone(), x.to_string());
                    }
                });
                ui.separator();
                ui.add_space(HORIZONTAL_GAP);
            });
        }
        match self.open_request_panel_enum {
            RequestPanelEnum::Params => {
                self.params_panel.set_and_render(app_data,cursor,ui)
            }
            RequestPanelEnum::Authorization => {}
            RequestPanelEnum::Headers => {}
            RequestPanelEnum::Body => {}
        }
        ui.separator();
    }
}