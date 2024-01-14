use std::collections::BTreeSet;

use eframe::emath::Align;
use egui::{Button, Layout, ScrollArea, Ui};

use crate::data::config_data::ConfigData;
use crate::data::cookies_manager::Cookie;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::panels::VERTICAL_GAP;
use crate::utils;

pub struct CookiesWindows {
    cookies_windows_open: bool,
    new_cookie_name: String,
    new_key_name: String,
    select_domain_name: Option<String>,
    select_key_name: Option<String>,
    select_content: String,
    new_cookie_names: BTreeSet<String>,
}

impl Default for CookiesWindows {
    fn default() -> Self {
        CookiesWindows {
            cookies_windows_open: false,
            new_cookie_name: "".to_string(),
            new_key_name: "".to_string(),
            select_domain_name: None,
            select_key_name: None,
            select_content: "".to_string(),
            new_cookie_names: Default::default(),
        }
    }
}

impl Window for CookiesWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("MANAGE COOKIES")
            .max_width(500.0)
            .min_height(400.0)
            .max_height(400.0)
            .collapsible(false)
            .resizable(true)
            .modal(true)
    }

    fn set_open(&mut self, open: bool) {
        self.cookies_windows_open = open;
    }

    fn get_open(&self) -> bool {
        self.cookies_windows_open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        _: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        ui.vertical(|ui| {
            ui.add_space(VERTICAL_GAP);
            self.render_add(workspace_data, ui);
            ui.add_space(VERTICAL_GAP);
            ui.separator();
            self.render_domain_list(workspace_data, ui);
            ui.add_space(VERTICAL_GAP);
            ui.separator();
            self.render_key_list(workspace_data, &operation, ui);
            ui.add_space(VERTICAL_GAP);
            ui.separator();
            self.render_content(workspace_data, &operation, ui);
        });
    }
}
impl CookiesWindows {
    fn render_add(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.label("Add a cookie domain.");
        ui.horizontal(|ui| {
            utils::text_edit_singleline_justify(ui, &mut self.new_cookie_name);
            if self.new_cookie_name == ""
                || workspace_data.cookies_contain_domain(self.new_cookie_name.clone())
            {
                ui.set_enabled(false);
            }
            if ui.button("Add").clicked() {
                self.new_cookie_names.insert(self.new_cookie_name.clone());
                self.new_cookie_name = "".to_string();
            }
            ui.set_enabled(true);
        });
    }

    fn render_domain_list(&mut self, workspace_data: &mut WorkspaceData, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut names = workspace_data.get_cookie_domains();
            for new_names in self.new_cookie_names.iter() {
                names.push(new_names.clone());
            }
            for name in names {
                let response = ui.selectable_value(
                    &mut self.select_domain_name,
                    Some(name.to_string()),
                    name.as_str(),
                );
                if response.clicked() {
                    self.select_key_name = None;
                }
                response.context_menu(|ui| {
                    if ui.button("Remove").clicked() {
                        workspace_data.remove_cookie_domain(name.clone());
                        self.new_cookie_names.remove(name.as_str());
                        ui.close_menu();
                    }
                });
            }
        });
    }

    fn render_key_list(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        ui: &mut Ui,
    ) {
        ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| match &self.select_domain_name {
                None => {}
                Some(domain) => {
                    ui.horizontal(|ui| {
                        if self.new_key_name == ""
                            || workspace_data.cookies_contain_domain_key(
                                domain.clone(),
                                self.new_key_name.clone(),
                            )
                        {
                            ui.add_enabled(false, Button::new("+"));
                        } else {
                            if ui.button("+").clicked() {
                                match workspace_data.add_domain_cookies(Cookie {
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
                                }) {
                                    Ok(_) => {
                                        self.new_cookie_names.remove(domain.as_str());
                                    }
                                    Err(err) => {
                                        operation.add_error_toast(err);
                                    }
                                }
                                self.new_key_name = "".to_string();
                            }
                        }
                        utils::text_edit_singleline_filter_justify(ui, &mut self.new_key_name);
                    });
                    let option_cookies = workspace_data.get_domain_cookies(domain.to_string());
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
                                            workspace_data.remove_cookie_domain_path_name(
                                                domain.to_string(),
                                                c.path.clone(),
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

    fn render_content(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
        ui: &mut Ui,
    ) {
        match &self.select_domain_name {
            None => {}
            Some(domain) => {
                let option_map = workspace_data.get_domain_cookies(domain.to_string());
                option_map.map(|map| match &self.select_key_name {
                    None => {}
                    Some(key) => {
                        let cookie = map.get(key);
                        cookie.map(|c| {
                            utils::text_edit_multiline_justify(ui, &mut self.select_content);
                            ui.add_space(VERTICAL_GAP);
                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                let new_cookie = Cookie::from_raw(self.select_content.clone());
                                if ui.button("Remove").clicked() {
                                    workspace_data.remove_cookie_domain_path_name(
                                        domain.clone(),
                                        c.path.clone(),
                                        key.clone(),
                                    );
                                }
                                if ui.button("Save").clicked() {
                                    match workspace_data.update_domain_cookies(
                                        new_cookie,
                                        c.domain.clone(),
                                        c.name.clone(),
                                    ) {
                                        Ok(_) => {
                                            operation.add_success_toast("Update cookie success.");
                                        }
                                        Err(err) => {
                                            operation.add_error_toast(err);
                                        }
                                    }
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
