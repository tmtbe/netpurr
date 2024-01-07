use eframe::emath::Align;
use egui::{FontSelection, Response, RichText, Style, Ui, WidgetText};
use uuid::Uuid;

use crate::data::central_request_data::CentralRequestItem;
use crate::data::environment::ENVIRONMENT_GLOBALS;
use crate::data::workspace::WorkspaceData;
use crate::operation::Operation;
use crate::panels::rest_panel::RestPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::utils;
use crate::windows::cookies_windows::CookiesWindows;
use crate::windows::environment_windows::EnvironmentWindows;
use crate::windows::new_collection_windows::NewCollectionWindows;
use crate::windows::request_close_windows::RequestCloseWindows;
use crate::windows::save_crt_windows::SaveCRTWindows;
use crate::windows::save_windows::SaveWindows;

#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel: RestPanel,
    environment_windows: EnvironmentWindows,
    save_windows: SaveWindows,
    new_collection_windows: NewCollectionWindows,
    cookies_windows: CookiesWindows,
    request_close_windows: RequestCloseWindows,
    save_crt_windows: SaveCRTWindows,
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
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        ui.horizontal(|ui| {
            self.central_environment(workspace_data, ui);
            self.central_request_table(workspace_data, ui);
        });
        ui.separator();
        match &workspace_data.central_request_data_list.select_id {
            Some(request_id) => {
                self.editor_panel
                    .set_and_render(ui, operation, workspace_data, request_id.clone());
            }
            _ => {}
        }
        self.environment_windows
            .set_and_render(ui, operation, workspace_data, cursor);
        workspace_data.environment.select().clone().map(|s| {
            if !workspace_data
                .environment
                .get_data()
                .contains_key(s.as_str())
            {
                workspace_data.environment.set_select(None)
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
        self.save_windows
            .set_and_render(ui, operation, workspace_data, 0);
        self.request_close_windows
            .set_and_render(ui, operation, workspace_data);
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
            .set_and_render(ui, operation, workspace_data, 0);
        if operation.open_windows().cookies_opened {
            self.cookies_windows.open();
            operation.open_windows().cookies_opened = false;
        }
        self.cookies_windows
            .set_and_render(ui, operation, workspace_data, 0);
        if operation.open_windows().save_crt_opened {
            self.save_crt_windows
                .open(operation.open_windows().crt_id.clone());
            operation.open_windows().save_crt_opened = false;
        }
        self.save_crt_windows
            .set_and_render(ui, operation, workspace_data, 0);
    }
}

impl MyCentralPanel {
    pub fn selectable_value(
        &mut self,
        ui: &mut Ui,
        workspace_data: &mut WorkspaceData,
        selected_value: Option<String>,
        text: impl Into<WidgetText>,
    ) -> Response {
        let mut response =
            ui.selectable_label(workspace_data.environment.select() == selected_value, text);
        if response.clicked() && workspace_data.environment.select() != selected_value {
            workspace_data.environment.set_select(selected_value);
            response.mark_changed();
        }
        response
    }

    fn central_environment(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        egui::SidePanel::right("central_right_environment_panel")
            .resizable(true)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(HORIZONTAL_GAP * 2.0);
                    egui::ComboBox::from_id_source("env")
                        .selected_text(
                            workspace_data
                                .environment
                                .select()
                                .unwrap_or("No Environment".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            self.selectable_value(ui, workspace_data, None, "No Environment");
                            for (name, _) in &workspace_data.environment.get_data() {
                                if name == ENVIRONMENT_GLOBALS {
                                    continue;
                                }
                                self.selectable_value(
                                    ui,
                                    workspace_data,
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

    fn central_request_table(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        egui::SidePanel::left("central_request_table_panel")
            .resizable(true)
            .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
            .show_inside(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let data_list = workspace_data.central_request_data_list.data_list.clone();
                    for request_id in data_list.iter() {
                        let request_data_option = workspace_data
                            .central_request_data_list
                            .data_map
                            .get(request_id)
                            .cloned();
                        match request_data_option {
                            None => {
                                continue;
                            }
                            Some(request_data) => {
                                let mut lb = utils::build_rest_ui_header(
                                    request_data.rest.clone(),
                                    Some(15),
                                    ui,
                                );
                                if request_data.is_modify() {
                                    let style = Style::default();
                                    RichText::new(" ●")
                                        .color(ui.visuals().warn_fg_color)
                                        .append_to(
                                            &mut lb,
                                            &style,
                                            FontSelection::Default,
                                            Align::Center,
                                        );
                                }
                                let response = ui.selectable_value(
                                    &mut workspace_data.central_request_data_list.select_id,
                                    Some(request_data.id.clone()),
                                    lb,
                                );
                                response.context_menu(|ui| {
                                    if ui.button("Duplicate Tab").clicked() {
                                        let mut duplicate = request_data.clone();
                                        duplicate.id = Uuid::new_v4().to_string();
                                        duplicate.collection_path = None;
                                        workspace_data.central_request_data_list.add_crt(duplicate);
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button("Close").clicked() {
                                        if !request_data.is_modify() {
                                            self.close_tab(workspace_data, &request_data);
                                        } else {
                                            self.request_close_windows.open(
                                                request_data.id.clone(),
                                                request_data.get_tab_name(),
                                            );
                                        }
                                        ui.close_menu();
                                    }
                                    if ui.button("Force Close").clicked() {
                                        self.close_tab(workspace_data, &request_data);
                                        ui.close_menu();
                                    }
                                    if ui.button("Force Close Other Tabs").clicked() {
                                        workspace_data.central_request_data_list.clear();
                                        workspace_data
                                            .central_request_data_list
                                            .add_crt(request_data.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("Force Close All Tabs").clicked() {
                                        workspace_data.central_request_data_list.clear();
                                        workspace_data.central_request_data_list.select_id = None;
                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    }
                    if ui.button("+").clicked() {
                        workspace_data.central_request_data_list.add_new()
                    }
                    if ui.button("...").clicked() {}
                });
            });
    }

    fn close_tab(&self, workspace_data: &mut WorkspaceData, request_data: &CentralRequestItem) {
        workspace_data
            .central_request_data_list
            .remove(request_data.id.clone());
        if workspace_data.central_request_data_list.select_id.is_some() {
            if workspace_data
                .central_request_data_list
                .select_id
                .clone()
                .unwrap()
                == request_data.id
            {
                workspace_data.central_request_data_list.select_id = None;
            }
        }
    }
}
