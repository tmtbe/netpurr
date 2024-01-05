use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use eframe::emath::Align2;
use eframe::epaint::ahash::HashMap;
use egui_toast::Toasts;
use log::info;
use poll_promise::Promise;
use urlencoding::encode;

use ehttp::Request;

use crate::data::{
    Collection, CollectionFolder, EnvironmentItemValue, Header, HttpRecord, Logger, QueryParam,
};
use crate::script::script::{Context, ScriptRuntime, ScriptScope};
use crate::{data, utils};

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
    pub fn send_with_script(
        &self,
        request: data::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        scripts: Vec<ScriptScope>,
    ) -> Promise<Result<(data::Request, ehttp::Response, Logger), String>> {
        Promise::spawn_thread("send_with_script", move || {
            let mut context_result = Ok(Context {
                scope_name: "".to_string(),
                request: request.clone(),
                envs: envs.clone(),
                shared_map: Default::default(),
                logger: Default::default(),
            });
            if scripts.len() > 0 {
                context_result = ScriptRuntime::run_block_many(
                    scripts,
                    Context {
                        scope_name: "".to_string(),
                        request: request.clone(),
                        envs: envs.clone(),
                        shared_map: Default::default(),
                        logger: Default::default(),
                    },
                );
            }
            let mut logger = data::Logger::default();
            match context_result {
                Ok(context) => {
                    logger = context.logger;
                    match RestSender::block_send(context.request.clone(), context.envs.clone()) {
                        Ok(response) => Ok((context.request, response, logger)),
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        })
    }
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
        request: data::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> Promise<ehttp::Result<ehttp::Response>> {
        let (sender, promise) = Promise::new();
        let request = Self::build_request(request, envs);
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });
        return promise;
    }

    pub fn block_send(
        request: data::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> ehttp::Result<ehttp::Response> {
        let request = Self::build_request(request.clone(), envs);
        ehttp::fetch_blocking(&request)
    }

    fn build_request(
        mut request: data::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> Request {
        if !request.base_url.starts_with("http://") && !request.base_url.starts_with("https://") {
            request.base_url = "http://".to_string() + request.base_url.as_str();
        }
        let content_type = request.body.build_body(&envs);
        content_type.map(|c| {
            request.set_request_content_type(c);
        });
        let headers = Self::build_header(&request, &envs);
        let request = Request {
            method: request.method.to_string(),
            url: Self::build_url(&request, &envs),
            body: request.body.to_vec(),
            headers,
        };
        info!("{:?}", request);
        request
    }

    fn build_header(
        request: &data::Request,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> Vec<(String, String)> {
        request
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
    fn build_url(request: &data::Request, envs: &BTreeMap<String, EnvironmentItemValue>) -> String {
        let url = utils::replace_variable(request.base_url.clone(), envs.clone());
        let params: Vec<String> = request
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
