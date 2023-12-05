use egui::{CollapsingHeader, RichText, Ui};

use crate::data::{AppData, CentralRequestItem};
use crate::panels::DataView;

#[derive(Default)]
pub struct HistoryPanel {}

impl DataView for HistoryPanel {
    type CursorType = i32;
    fn set_and_render(&mut self,app_data: &mut AppData, cursor: Self::CursorType, ui: &mut egui::Ui){
        for date_history_data in app_data.history_data_list.date_group_list.iter_mut() {
            CollapsingHeader::new(date_history_data.date.clone())
                .default_open(false)
                .show(ui, |ui| {
                    for history_rest_item in date_history_data.history_list.iter() {
                        if ui.button(RichText::new(history_rest_item.rest.clone().request.method.to_string() + &*history_rest_item.rest.clone().request.url)
                            .color(ui.visuals().warn_fg_color)).clicked() {
                            app_data.central_request_data_list.add_crt(CentralRequestItem{
                                id: history_rest_item.id.clone(),
                                rest: history_rest_item.rest.clone(),
                            })
                        }
                    }
                });
        }
    }
}