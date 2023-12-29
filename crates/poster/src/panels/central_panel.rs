use egui::{Response, Ui, WidgetText};
use uuid::Uuid;

use crate::data::{AppData, CentralRequestItem, ENVIRONMENT_GLOBALS};
use crate::operation::Operation;
use crate::panels::cookies_windows::CookiesWindows;
use crate::panels::environment_windows::EnvironmentWindows;
use crate::panels::new_collection_windows::NewCollectionWindows;
use crate::panels::rest_panel::RestPanel;
use crate::panels::save_windows::SaveWindows;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel: RestPanel,
    environment_windows: EnvironmentWindows,
    save_windows: SaveWindows,
    new_collection_windows: NewCollectionWindows,
    cookies_windows: CookiesWindows,
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
        ui: &mut Ui,
        operation: &mut Operation,
        app_data: &mut AppData,
        cursor: Self::CursorType,
    ) {
        ui.horizontal(|ui| {
            self.central_environment(app_data, ui);
            self.central_request_table(app_data, ui);
        });
        ui.separator();
        match &app_data.central_request_data_list.select_id {
            Some(request_id) => {
                self.editor_panel
                    .set_and_render(ui, operation, app_data, request_id.clone());
            }
            _ => {}
        }
        self.environment_windows
            .set_and_render(ui, operation, app_data, cursor);
        app_data.environment.select().clone().map(|s| {
            if !app_data.environment.get_data().contains_key(s.as_str()) {
                app_data.environment.set_select(None)
            }
        });
        if operation.open_windows().save_opened {
            self.save_windows.open(
                operation.open_windows().http_record.clone(),
                operation.open_windows().default_path.clone(),
                operation.open_windows().edit,
            );
            operation.open_windows().save_opened = false;
        }
        self.save_windows.set_and_render(ui, operation, app_data, 0);
        if operation.open_windows().collection_opened {
            self.new_collection_windows
                .open_collection(operation.open_windows().collection.clone());
            operation.open_windows().collection_opened = false;
        }
        if operation.open_windows().folder_opened {
            self.new_collection_windows.open_folder(
                operation.open_windows().collection.clone().unwrap(),
                operation.open_windows().parent_folder.clone(),
                operation.open_windows().folder.clone(),
            );
            operation.open_windows().folder_opened = false;
        }
        self.new_collection_windows
            .set_and_render(ui, operation, app_data, 0);
        if operation.open_windows().cookies_opened {
            self.cookies_windows.open();
            operation.open_windows().cookies_opened = false;
        }
        self.cookies_windows
            .set_and_render(ui, operation, app_data, 0);
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

    fn central_environment(&mut self, app_data: &mut AppData, ui: &mut Ui) {
        egui::SidePanel::right("central_right_environment_panel")
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
    }

    fn central_request_table(&self, app_data: &mut AppData, ui: &mut Ui) {
        egui::SidePanel::left("central_request_table_panel")
            .resizable(true)
            .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
            .show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for request_data in app_data.central_request_data_list.data_list.clone().iter()
                    {
                        let lb =
                            utils::build_rest_ui_header(request_data.rest.clone(), Some(15), ui);
                        let response = ui.selectable_value(
                            &mut app_data.central_request_data_list.select_id,
                            Some(request_data.id.clone()),
                            lb,
                        );
                        response.context_menu(|ui| {
                            if ui.button("Duplicate Tab").clicked() {
                                let mut duplicate = request_data.clone();
                                duplicate.id = Uuid::new_v4().to_string();
                                app_data.central_request_data_list.add_crt(duplicate);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Close").clicked() {
                                self.close_tab(app_data, request_data);
                                ui.close_menu();
                            }
                            if ui.button("Force Close").clicked() {
                                self.close_tab(app_data, request_data);
                                ui.close_menu();
                            }
                            if ui.button("Close Other Tabs").clicked() {
                                app_data.central_request_data_list.clear();
                                app_data
                                    .central_request_data_list
                                    .add_crt(request_data.clone());
                                ui.close_menu();
                            }
                            if ui.button("Close All Tabs").clicked() {
                                app_data.central_request_data_list.clear();
                                app_data.central_request_data_list.select_id = None;
                                ui.close_menu();
                            }
                            if ui.button("Force Close All Tabs").clicked() {
                                app_data.central_request_data_list.clear();
                                app_data.central_request_data_list.select_id = None;
                                ui.close_menu();
                            }
                        });
                    }
                    if ui.button("+").clicked() {
                        app_data.central_request_data_list.add_new()
                    }
                    if ui.button("...").clicked() {}
                });
            });
    }

    fn close_tab(&self, app_data: &mut AppData, request_data: &CentralRequestItem) {
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
