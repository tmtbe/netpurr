use std::cell::RefCell;
use std::rc::Rc;

use egui::{Context, Ui};

use crate::data::collections::{Collection, CollectionFolder};
use crate::data::config_data::ConfigData;
use crate::data::http::HttpRecord;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;

pub trait Window {
    fn window_setting(&self) -> WindowSetting;
    fn set_open(&mut self, open: bool);
    fn get_open(&self) -> bool;
    fn render(
        &mut self,
        ui: &mut Ui,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    );
}

#[derive(Default, Clone)]
pub struct WindowSetting {
    name: String,
    collapsible: bool,
    resizable: bool,
    default_width: Option<f32>,
    default_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    min_width: Option<f32>,
    min_height: Option<f32>,
    modal: bool,
}

impl WindowSetting {
    pub fn new(name: String) -> Self {
        WindowSetting {
            name,
            ..Default::default()
        }
    }
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }
    pub fn default_height(mut self, default_height: f32) -> Self {
        self.default_height = Some(default_height);
        self
    }
    pub fn default_width(mut self, default_width: f32) -> Self {
        self.default_width = Some(default_width);
        self
    }
}

#[derive(Default)]
pub struct Windows {
    show_windows: Vec<Box<dyn Window>>,
}

impl Windows {
    pub fn add(&mut self, mut window: Box<dyn Window>) {
        window.set_open(true);
        self.show_windows.push(window);
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        for window in self.show_windows.iter_mut() {
            let mut open = window.get_open();
            if window.window_setting().modal {
                operation.lock_ui(window.window_setting().name.clone(), true);
            }
            let mut w = egui::Window::new(window.window_setting().name.clone());
            if let Some(v) = window.window_setting().max_width {
                w = w.max_width(v)
            }
            if let Some(v) = window.window_setting().min_width {
                w = w.min_width(v)
            }
            if let Some(v) = window.window_setting().max_height {
                w = w.max_height(v)
            }
            if let Some(v) = window.window_setting().min_height {
                w = w.min_height(v)
            }
            if let Some(v) = window.window_setting().default_width {
                w = w.default_width(v)
            }
            if let Some(v) = window.window_setting().default_height {
                w = w.default_height(v)
            }
            w.collapsible(window.window_setting().collapsible)
                .resizable(window.window_setting().resizable)
                .open(&mut open)
                .show(ctx, |ui| {
                    window.render(ui, config_data, workspace_data, operation.clone())
                });
            open = window.get_open();
            if !open {
                operation.lock_ui(window.window_setting().name.clone(), false);
            }
        }
        self.show_windows.retain(|w| w.get_open())
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct OpenWindows {
    pub save_opened: bool,
    pub edit: bool,
    pub collection_opened: bool,
    pub folder_opened: bool,
    pub cookies_opened: bool,
    pub http_record: HttpRecord,
    pub default_path: Option<String>,
    pub collection: Option<Collection>,
    pub parent_folder: Rc<RefCell<CollectionFolder>>,
    pub folder: Option<Rc<RefCell<CollectionFolder>>>,
    pub crt_id: String,
    pub save_crt_opened: bool,
}
