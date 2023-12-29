use std::collections::BTreeMap;

use eframe::emath::Align;
use egui::{Button, Layout, ScrollArea, Ui};

use crate::data::{Cookie, WorkspaceData};
use crate::operation::Operation;
use crate::panels::{DataView, VERTICAL_GAP};
use crate::utils;

#[derive(Default)]
pub struct CookiesWindows {
    cookies_windows_open: bool,
    new_cookie_name: String,
    new_key_name: String,
    select_domain_name: Option<String>,
    select_key_name: Option<String>,
    select_content: String,
}

impl CookiesWindows {
    pub(crate) fn open(&mut self) {
        self.cookies_windows_open = true;
    }

    fn render_add(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.label("Add a cookie domain.");
        ui.horizontal(|ui| {
            utils::text_edit_singleline_filter_justify(ui, &mut self.new_cookie_name);
            if self.new_cookie_name == ""
                || workspace_data
                    .cookies_manager
                    .contain_domain(self.new_cookie_name.clone())
            {
                ui.set_enabled(false);
            }
            if ui.button("Add").clicked() {
                workspace_data
                    .cookies_manager
                    .set_domain_cookies(self.new_cookie_name.clone(), BTreeMap::new());
                self.new_cookie_name = "".to_string();
            }
            ui.set_enabled(true);
        });
    }

    fn render_domain_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let names = workspace_data.cookies_manager.get_cookies_names();
            for name in names {
                let response = ui.selectable_value(
                    &mut self.select_domain_name,
                    Some(name.to_string()),
                    name.as_str(),
                );
                response.context_menu(|ui| {
                    if ui.button("Remove").clicked() {
                        workspace_data.cookies_manager.remove_domain(name.clone());
                        ui.close_menu();
                    }
                });
            }
        });
    }

    fn render_key_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| match &self.select_domain_name {
                None => {}
                Some(domain) => {
                    ui.horizontal(|ui| {
                        if self.new_key_name == ""
                            || workspace_data
                                .cookies_manager
                                .contain_domain_key(domain.clone(), self.new_key_name.clone())
                        {
                            ui.add_enabled(false, Button::new("+"));
                        } else {
                            if ui.button("+").clicked() {
                                workspace_data.cookies_manager.add_domain_cookies(
                                    domain.to_string(),
                                    self.new_key_name.to_string(),
                                    Cookie {
                                        name: self.new_key_name.clone(),
                                        value: "NONE".to_string(),
                                        domain: domain.clone(),
                                        path: "/".to_string(),
                                        expires: "".to_string(),
                                        max_age: "".to_string(),
                                        raw: format!(
                                            "{}={}; path=/; domain={}",
                                            self.new_key_name, "NONE", domain
                                        ),
                                        http_only: false,
                                        secure: false,
                                    },
                                );
                                self.new_key_name = "".to_string();
                            }
                        }
                        utils::text_edit_singleline_filter_justify(ui, &mut self.new_key_name);
                    });
                    let option_cookies = workspace_data
                        .cookies_manager
                        .get_domain_cookies(domain.to_string());
                    match option_cookies {
                        None => {}
                        Some(cookies) => {
                            for (name, c) in cookies.iter() {
                                ui.vertical(|ui| {
                                    let response = utils::select_value(
                                        ui,
                                        &mut self.select_key_name,
                                        Some(name.clone()),
                                        name,
                                    );
                                    if response.clicked() {
                                        self.select_domain_name.clone().map(|domain| {
                                            self.select_content = c.raw.clone();
                                        });
                                    }
                                    response.context_menu(|ui| {
                                        if ui.button("Remove").clicked() {
                                            workspace_data.cookies_manager.remove_domain_key(
                                                domain.to_string(),
                                                name.clone(),
                                            );
                                            ui.close_menu();
                                        }
                                    });
                                });
                            }
                        }
                    }
                }
            });
    }

    fn render_content(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        match &self.select_domain_name {
            None => {}
            Some(domain) => {
                let option_map = workspace_data
                    .cookies_manager
                    .get_domain_cookies(domain.to_string());
                option_map.map(|map| match &self.select_key_name {
                    None => {}
                    Some(key) => {
                        let cookie = map.get(key);
                        cookie.map(|c| {
                            utils::text_edit_multiline_justify(ui, &mut self.select_content);
                            ui.add_space(VERTICAL_GAP);
                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                let new_cookie = Cookie::from_raw(self.select_content.clone());
                                if new_cookie.name != c.name || new_cookie.domain != c.domain {
                                    ui.set_enabled(false);
                                }
                                if ui.button("Save").clicked() {
                                    workspace_data.cookies_manager.update_domain_key(
                                        domain.to_string(),
                                        key.to_string(),
                                        new_cookie,
                                    );
                                }
                                if ui.button("Remove").clicked() {
                                    workspace_data
                                        .cookies_manager
                                        .remove_domain_key(domain.to_string(), key.to_string());
                                }
                                ui.set_enabled(true);
                            });
                        });
                    }
                });
            }
        }
    }
}

impl DataView for CookiesWindows {
    type CursorType = i32;

    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        operation.lock_ui("env".to_string(), self.cookies_windows_open);
        let mut cookies_windows_open = self.cookies_windows_open;
        egui::Window::new("MANAGE COOKIES")
            .default_open(true)
            .max_width(500.0)
            .min_height(400.0)
            .max_height(400.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut cookies_windows_open)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.add_space(VERTICAL_GAP);
                    self.render_add(workspace_data, ui);
                    ui.add_space(VERTICAL_GAP);
                    ui.separator();
                    self.render_domain_list(workspace_data, ui);
                    ui.add_space(VERTICAL_GAP);
                    ui.separator();
                    self.render_key_list(workspace_data, ui);
                    ui.add_space(VERTICAL_GAP);
                    ui.separator();
                    self.render_content(workspace_data, ui);
                });
            });
        self.cookies_windows_open = cookies_windows_open;
    }
}
