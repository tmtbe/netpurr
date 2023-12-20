use egui::{Response, Ui, WidgetText};

use crate::data::{AppData, ENVIRONMENT_GLOBALS};
use crate::panels::environment_windows::EnvironmentWindows;
use crate::panels::rest_panel::RestPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel: RestPanel,
    environment_windows: EnvironmentWindows,
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
        ui.horizontal(|ui| {
            egui::SidePanel::right("central_right_panel")
                .resizable(true)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(HORIZONTAL_GAP * 2.0);
                        egui::ComboBox::from_id_source("env")
                            .selected_text(
                                app_data
                                    .environment
                                    .select()
                                    .unwrap_or("No Environment".to_string()),
                            )
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                self.selectable_value(ui, app_data, None, "No Environment");
                                for (name, _) in &app_data.environment.get_data() {
                                    if name == ENVIRONMENT_GLOBALS {
                                        continue;
                                    }
                                    self.selectable_value(
                                        ui,
                                        app_data,
                                        Some(name.clone()),
                                        name.clone(),
                                    );
                                }
                            });
                        if ui.button("⚙").clicked() {
                            self.environment_windows.open()
                        }
                    });
                });
            egui::SidePanel::left("central_left_panel")
                .resizable(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for request_data in
                            app_data.central_request_data_list.data_list.clone().iter()
                        {
                            let lb = utils::build_rest_ui_header(request_data.rest.clone(), ui);
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
                });
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
        self.environment_windows
            .set_and_render(app_data, cursor, ui);
        app_data.environment.select().clone().map(|s| {
            if !app_data.environment.get_data().contains_key(s.as_str()) {
                app_data.environment.set_select(None)
            }
        });
    }
}

impl MyCentralPanel {
    pub fn selectable_value(
        &mut self,
        ui: &mut Ui,
        app_data: &mut AppData,
        selected_value: Option<String>,
        text: impl Into<WidgetText>,
    ) -> Response {
        let mut response =
            ui.selectable_label(app_data.environment.select() == selected_value, text);
        if response.clicked() && app_data.environment.select() != selected_value {
            app_data.environment.set_select(selected_value);
            response.mark_changed();
        }
        response
    }
}
