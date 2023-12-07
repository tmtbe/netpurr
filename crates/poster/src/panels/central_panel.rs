use crate::data::AppData;
use crate::panels::rest_panel::RestPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel: RestPanel,
}

#[derive(PartialEq, Eq, Clone)]
enum PanelEnum {
    RequestId(Option<String>),
}

impl Default for PanelEnum {
    fn default() -> Self {
        PanelEnum::RequestId(None)
    }
}

impl DataView for MyCentralPanel {
    type CursorType = i32;
    fn set_and_render(
        &mut self,
        app_data: &mut AppData,
        cursor: Self::CursorType,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal_wrapped(|ui| {
            for request_data in app_data.central_request_data_list.data_list.clone().iter() {
                let lb = utils::build_rest_ui_header(request_data.rest.request.clone(), ui);
                if ui
                    .selectable_value(
                        &mut app_data.central_request_data_list.select_id,
                        Some(request_data.id.clone()),
                        lb,
                    )
                    .double_clicked()
                {
                    app_data
                        .central_request_data_list
                        .remove(request_data.id.clone());
                    if app_data.central_request_data_list.select_id.is_some() {
                        if app_data
                            .central_request_data_list
                            .select_id
                            .clone()
                            .unwrap()
                            == request_data.id
                        {
                            app_data.central_request_data_list.select_id = None;
                        }
                    }
                }
            }
            if ui.button("+").clicked() {
                app_data.central_request_data_list.add_new()
            }
            if ui.button("...").clicked() {}
        });
        ui.separator();
        ui.add_space(HORIZONTAL_GAP);
        match &app_data.central_request_data_list.select_id {
            Some(request_id) => {
                self.editor_panel
                    .set_and_render(app_data, request_id.clone(), ui);
            }
            _ => {}
        }
    }
}
