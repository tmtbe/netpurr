use std::cell::RefCell;
use std::rc::Rc;

use egui::{Context, Ui};

use crate::data::collections::{Collection, CollectionFolder};
use crate::data::http::HttpRecord;
use crate::data::workspace_data::WorkspaceData;

pub trait Window {
    fn window_setting(&self) -> &WindowSetting;
    fn set_open(&mut self, open: bool);
    fn get_open(&self) -> bool;
    fn render(&mut self, ui: &mut Ui, workspace_data: &mut WorkspaceData);
}

#[derive(Default, Clone)]
pub struct WindowSetting {
    name: String,
    collapsible: bool,
    resizable: bool,
    max_width: f32,
    max_height: f32,
    min_width: f32,
    min_height: f32,
}

impl WindowSetting {
    pub fn new(name: String) -> Self {
        WindowSetting {
            name,
            ..Default::default()
        }
    }
    pub fn collapsible(&mut self, collapsible: bool) -> &Self {
        self.collapsible = collapsible;
        self
    }
    pub fn resizable(&mut self, resizable: bool) -> &Self {
        self.resizable = resizable;
        self
    }
    pub fn max_width(&mut self, max_width: f32) -> &Self {
        self.max_width = max_width;
        self
    }
    pub fn max_height(&mut self, max_height: f32) -> &Self {
        self.max_height = max_height;
        self
    }
    pub fn min_height(&mut self, min_height: f32) -> &Self {
        self.min_height = min_height;
        self
    }
    pub fn min_width(&mut self, min_width: f32) -> &Self {
        self.min_width = min_width;
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

    pub fn show(&mut self, ctx: &Context, workspace_data: &mut WorkspaceData) {
        for window in self.show_windows.iter_mut() {
            let mut open = window.get_open();
            egui::Window::new(window.window_setting().name.clone())
                .default_open(true)
                .max_width(window.window_setting().max_width)
                .min_height(window.window_setting().min_height)
                .max_height(window.window_setting().max_height)
                .collapsible(window.window_setting().collapsible)
                .resizable(window.window_setting().resizable)
                .open(&mut open)
                .show(ctx, |ui| window.render(ui, workspace_data));
            window.set_open(open);
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

impl OpenWindows {
    pub fn open_crt_save(&mut self, crt_id: String) {
        self.crt_id = crt_id;
        self.save_crt_opened = true;
    }
    pub fn open_save(&mut self, http_record: HttpRecord, default_path: Option<String>) {
        self.http_record = http_record;
        self.default_path = default_path;
        self.save_opened = true;
        self.edit = false;
    }
    pub fn open_edit(&mut self, http_record: HttpRecord, default_path: String) {
        self.http_record = http_record;
        self.default_path = Some(default_path);
        self.save_opened = true;
        self.edit = true
    }
    pub fn open_collection(&mut self, collection: Option<Collection>) {
        self.collection = collection;
        self.collection_opened = true;
    }
    pub fn open_folder(
        &mut self,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Option<Rc<RefCell<CollectionFolder>>>,
    ) {
        self.collection = Some(collection);
        self.parent_folder = parent_folder;
        self.folder = folder;
        self.folder_opened = true;
    }

    pub fn open_cookies(&mut self) {
        self.cookies_opened = true
    }
}
