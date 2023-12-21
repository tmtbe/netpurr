use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use egui::{Ui, Widget};
use strum::IntoEnumIterator;

use crate::data::{Auth, AuthType, Collection, CollectionFolder, EnvironmentItemValue};
use crate::panels::{AlongDataView, HORIZONTAL_GAP, VERTICAL_GAP};
use crate::widgets::highlight_template_singleline::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct AuthPanel {
    envs: BTreeMap<String, EnvironmentItemValue>,
    collection: Option<Collection>,
    folder: Option<Rc<RefCell<CollectionFolder>>>,
    parent_auth: Option<Auth>,
    name: String,
    no_inherit: bool,
}

impl AuthPanel {
    pub fn set_envs(
        &mut self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        parent_auth: Option<Auth>,
    ) {
        self.envs = envs;
        self.parent_auth = parent_auth
    }

    pub fn set_collection_folder(
        &mut self,
        collection: Collection,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        self.envs = collection.build_envs();
        self.collection = Some(collection);
        self.name = folder.borrow().name.clone();
        self.no_inherit = folder.borrow().is_root;
        self.folder = Some(folder);
    }
}

impl AlongDataView for AuthPanel {
    type DataType = Auth;

    fn set_and_render(&mut self, data: &mut Self::DataType, ui: &mut Ui) {
        ui.horizontal(|ui| {
            egui::SidePanel::left(self.name.clone() + "auth_left")
                .resizable(true)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.strong("AUTH");
                    ui.add_space(VERTICAL_GAP);
                    ui.label("The authorization header will be automatically generated when you send the request. ");
                    ui.add_space(VERTICAL_GAP);
                    egui::ComboBox::from_id_source("auth_type")
                        .selected_text(data.auth_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            for x in AuthType::iter() {
                                if self.no_inherit && x == AuthType::InheritAuthFromParent {
                                    continue;
                                }
                                ui.selectable_value(&mut data.auth_type, x.clone(), x.to_string());
                            }
                        });
                    ui.add_space(VERTICAL_GAP);
                });
            egui::SidePanel::right(self.name.clone() + "auth_right")
                .resizable(true)
                .show_separator_line(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| match data.auth_type {
                    AuthType::NoAuth => {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(VERTICAL_GAP * 5.0);
                            ui.label("This request does not use any authorization. ");
                            ui.add_space(VERTICAL_GAP * 5.0);
                        });
                    }
                    AuthType::BearerToken => {
                        ui.add_space(VERTICAL_GAP * 5.0);
                        ui.horizontal(|ui| {
                            ui.add_space(HORIZONTAL_GAP);
                            ui.label("Token:");
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(self.envs.clone())
                                .build("token".to_string(), &mut data.bearer_token)
                                .ui(ui);
                            ui.add_space(HORIZONTAL_GAP);
                        });
                        ui.add_space(VERTICAL_GAP * 5.0);
                    }
                    AuthType::BasicAuth => {
                        ui.add_space(VERTICAL_GAP * 2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(HORIZONTAL_GAP);
                            ui.label("Username:");
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(self.envs.clone())
                                .build("username".to_string(), &mut data.basic_username)
                                .ui(ui);
                        });
                        ui.add_space(VERTICAL_GAP);
                        ui.horizontal(|ui| {
                            ui.add_space(HORIZONTAL_GAP);
                            ui.label("Password: ");
                            HighlightTemplateSinglelineBuilder::default()
                                .envs(self.envs.clone())
                                .build("password".to_string(), &mut data.basic_password)
                                .ui(ui);
                        });
                        ui.add_space(VERTICAL_GAP * 2.0);
                    }
                    AuthType::InheritAuthFromParent => {
                        ui.add_space(VERTICAL_GAP);
                        ui.label("This request is not inheriting any authorization helper at the moment. Save it in a collection to use the parent's authorization helper.");
                        ui.add_space(VERTICAL_GAP);
                        self.parent_auth.clone().map(|parent_auth| {
                            match parent_auth.auth_type {
                                AuthType::InheritAuthFromParent => {}
                                AuthType::NoAuth => {
                                    ui.centered_and_justified(|ui| {
                                        ui.add_space(VERTICAL_GAP * 5.0);
                                        ui.label("This request does not use any authorization. ");
                                        ui.add_space(VERTICAL_GAP * 5.0);
                                    });
                                }
                                AuthType::BearerToken => {
                                    ui.add_space(VERTICAL_GAP * 5.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(HORIZONTAL_GAP);
                                        ui.label("Token:");
                                        ui.label(parent_auth.bearer_token.clone());
                                        ui.add_space(HORIZONTAL_GAP);
                                    });
                                    ui.add_space(VERTICAL_GAP * 5.0);
                                }
                                AuthType::BasicAuth => {
                                    ui.add_space(VERTICAL_GAP * 2.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(HORIZONTAL_GAP);
                                        ui.label("Username:");
                                        ui.label(parent_auth.basic_username.clone());
                                    });
                                    ui.add_space(VERTICAL_GAP);
                                    ui.horizontal(|ui| {
                                        ui.add_space(HORIZONTAL_GAP);
                                        ui.label("Password: ");
                                        ui.label(parent_auth.basic_password.clone());
                                    });
                                    ui.add_space(VERTICAL_GAP * 2.0);
                                }
                            }
                        });
                    }
                });
        });
    }
}
