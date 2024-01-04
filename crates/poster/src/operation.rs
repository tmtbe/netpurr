use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use eframe::emath::Align2;
use eframe::epaint::ahash::HashMap;
use egui_toast::Toasts;
use poll_promise::Promise;
use urlencoding::encode;

use crate::data::{
    Collection, CollectionFolder, EnvironmentItemValue, Header, HttpRecord, QueryParam,
};
use crate::script::script::ScriptRuntime;
use crate::utils;

pub struct Operation {
    rest_sender: RestSender,
    open_windows: OpenWindows,
    lock_ui: HashMap<String, bool>,
    script_runtime: ScriptRuntime,
    toasts: Toasts,
}

impl Default for Operation {
    fn default() -> Self {
        Operation {
            rest_sender: Default::default(),
            open_windows: Default::default(),
            lock_ui: Default::default(),
            script_runtime: Default::default(),
            toasts: Toasts::default()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(egui::Direction::BottomUp),
        }
    }
}
impl Operation {
    pub fn lock_ui(&mut self, key: String, bool: bool) {
        self.lock_ui.insert(key, bool);
    }
    pub fn get_ui_lock(&self) -> bool {
        let mut result = false;
        for (_, lock) in self.lock_ui.iter() {
            result = result || (lock.clone());
        }
        result
    }
    pub fn rest_sender(&self) -> &RestSender {
        &self.rest_sender
    }
    pub fn open_windows(&mut self) -> &mut OpenWindows {
        &mut self.open_windows
    }
    pub fn script_runtime(&self) -> &ScriptRuntime {
        &self.script_runtime
    }
    pub fn toasts(&mut self) -> &mut Toasts {
        &mut self.toasts
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}

impl RestSender {
    pub fn send(
        &self,
        rest: &mut HttpRecord,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> Promise<ehttp::Result<ehttp::Response>> {
        let (sender, promise) = Promise::new();
        if !rest.request.base_url.starts_with("http://")
            && !rest.request.base_url.starts_with("https://")
        {
            rest.request.base_url = "http://".to_string() + rest.request.base_url.as_str();
        }
        let content_type = rest.request.body.build_body(&envs);
        content_type.map(|c| {
            rest.set_request_content_type(c);
        });
        let headers = self.build_header(rest, &envs);
        let request = ehttp::Request {
            method: rest.request.method.to_string(),
            url: self.build_url(&rest, envs.clone()),
            body: rest.request.body.to_vec(),
            headers,
        };
        println!("{:?}", request);
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });
        return promise;
    }

    fn build_header(
        &self,
        rest: &mut HttpRecord,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> Vec<(String, String)> {
        rest.request
            .headers
            .iter()
            .filter(|h| h.enable)
            .map(|h| Header {
                key: h.key.clone(),
                value: utils::replace_variable(h.value.clone(), envs.clone()),
                desc: h.desc.clone(),
                enable: h.enable,
                lock_with: h.lock_with.clone(),
            })
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect()
    }
    fn build_url(&self, rest: &HttpRecord, envs: BTreeMap<String, EnvironmentItemValue>) -> String {
        let url = utils::replace_variable(rest.request.base_url.clone(), envs.clone());
        let params: Vec<String> = rest
            .request
            .params
            .iter()
            .filter(|p| p.enable)
            .map(|p| QueryParam {
                key: p.key.clone(),
                value: utils::replace_variable(p.value.clone(), envs.clone()),
                desc: p.desc.clone(),
                lock_with: p.lock_with.clone(),
                enable: p.enable,
            })
            .map(|p| format!("{}={}", encode(p.key.as_str()), encode(p.value.as_str())))
            .collect();
        url + "?" + params.join("&").as_str()
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
}

impl OpenWindows {
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
