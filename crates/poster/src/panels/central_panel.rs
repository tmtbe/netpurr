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
        utils::left_right_panel(
            ui,
            "central_left_panel",
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    for request_data in app_data.central_request_data_list.data_list.clone().iter()
                    {
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
            },
            "central_right_panel",
            |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP * 2.0);
                    egui::ComboBox::from_id_source("env")
                        .selected_text(
                            app_data
                                .environment
                                .select
                                .clone()
                                .unwrap_or("No Environment".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            ui.selectable_value(
                                &mut app_data.environment.select,
                                None,
                                "No Environment",
                            );
                            for (name, _) in &app_data.environment.data {
                                ui.selectable_value(
                                    &mut app_data.environment.select,
                                    Some(name.clone()),
                                    name.clone(),
                                );
                            }
                        });
                    if ui.button("⚙").clicked() {}
                });
            },
        );

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
