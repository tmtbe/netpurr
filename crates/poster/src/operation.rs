use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use eframe::emath::Align2;
use egui_toast::Toasts;
use poll_promise::Promise;
use reqwest::blocking::{multipart, Client};
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;

use crate::data::{
    BodyRawType, BodyType, Collection, CollectionFolder, EnvironmentItemValue, Header, HttpBody,
    HttpRecord, LockWith, Logger, MultipartDataType,
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
    ) -> Promise<Result<(data::Request, data::Response), String>> {
        let mut logger = Logger::default();
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
            match context_result {
                Ok(context) => {
                    for log in context.logger.logs.iter() {
                        logger.logs.push(log.clone());
                    }
                    let build_request =
                        RestSender::build_request(context.request.clone(), context.envs.clone());
                    logger.add_info(
                        "fetch".to_string(),
                        format!("start fetch request: {:?}", build_request),
                    );
                    match RestSender::reqwest_block_send(build_request) {
                        Ok((after_request, response)) => {
                            let mut after_response = response;
                            logger.add_info(
                                "fetch".to_string(),
                                format!("get response: {:?}", after_response),
                            );
                            after_response.logger = logger;
                            Ok((after_request, after_response))
                        }
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
    pub fn reqwest_block_send(
        request: data::Request,
    ) -> reqwest::Result<(data::Request, data::Response)> {
        let client = Client::new();
        let reqwest_request = Self::build_reqwest_request(request.clone())?;
        let mut new_request = request.clone();
        for (hn, hv) in reqwest_request.headers().iter() {
            if new_request
                .headers
                .iter()
                .find(|h| {
                    h.key.to_lowercase() == hn.to_string().to_lowercase()
                        && h.value == hv.to_str().unwrap()
                })
                .is_none()
            {
                new_request.headers.push(Header {
                    key: hn.to_string(),
                    value: hv.to_str().unwrap().to_string(),
                    desc: "auto gen".to_string(),
                    enable: true,
                    lock_with: LockWith::LockWithAuto,
                })
            }
        }
        let reqwest_response = client.execute(reqwest_request)?;
        Ok((
            new_request,
            data::Response {
                headers: Header::new_from_map(reqwest_response.headers()),
                status: reqwest_response.status().as_u16(),
                status_text: reqwest_response.status().to_string(),
                elapsed_time: 0,
                logger: Logger::default(),
                body: Arc::new(HttpBody::new(reqwest_response.bytes()?.to_vec())),
            },
        ))
    }

    pub fn build_reqwest_request(
        request: data::Request,
    ) -> reqwest::Result<reqwest::blocking::Request> {
        let client = Client::new();
        let method = Method::from_str(request.method.to_string().to_uppercase().as_str()).unwrap();
        let mut builder = client.request(method, request.base_url);
        for header in request.headers.iter().filter(|h| h.enable) {
            builder = builder.header(header.key.clone(), header.value.clone());
        }
        let query: Vec<(String, String)> = request
            .params
            .iter()
            .filter(|q| q.enable)
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect();
        builder = builder.query(&query);
        match request.body.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                let mut form = multipart::Form::new();
                for md in request.body.body_form_data.iter().filter(|md| md.enable) {
                    match md.data_type {
                        MultipartDataType::File => {
                            form = form
                                .file(md.key.clone(), Path::new(md.value.as_str()).to_path_buf())
                                .unwrap();
                        }
                        MultipartDataType::Text => {
                            form = form.text(md.key.clone(), md.value.clone());
                        }
                    }
                }
                builder = builder.multipart(form);
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                let mut params = HashMap::new();
                for md in request.body.body_xxx_form.iter().filter(|md| md.enable) {
                    params.insert(md.key.clone(), md.value.clone());
                }
                builder = builder.form(&params);
            }
            BodyType::RAW => match request.body.body_raw_type {
                BodyRawType::TEXT => {
                    builder = builder.header(CONTENT_TYPE, "text/plain");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::JSON => {
                    builder = builder.header(CONTENT_TYPE, "application/json");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::HTML => {
                    builder = builder.header(CONTENT_TYPE, "text/html");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::XML => {
                    builder = builder.header(CONTENT_TYPE, "application/xml");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::JavaScript => {
                    builder = builder.header(CONTENT_TYPE, "application/javascript");
                    builder = builder.body(request.body.body_str);
                }
            },
            BodyType::BINARY => {
                let path = Path::new(request.body.body_file.as_str());
                let content_type = mime_guess::from_path(path);
                builder = builder.header(
                    CONTENT_TYPE,
                    content_type.first_or_octet_stream().to_string(),
                );
                let file_name = path.file_name().and_then(|filename| filename.to_str());
                let mut file =
                    File::open(path).expect(format!("open {:?} error", file_name).as_str());
                let mut inner: Vec<u8> = vec![];
                io::copy(&mut file, &mut inner).expect("add_stream io copy error");
                builder = builder.body(inner);
            }
        }
        builder.build()
    }

    fn build_request(
        request: data::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> data::Request {
        let mut build_request = request.clone();
        if !build_request.base_url.starts_with("http://")
            && !build_request.base_url.starts_with("https://")
        {
            build_request.base_url = "http://".to_string() + build_request.base_url.as_str();
        }
        build_request.headers = Self::build_header(request.headers.clone(), &envs);
        build_request.body.body_str =
            utils::replace_variable(build_request.body.body_str, envs.clone());
        for md in build_request.body.body_xxx_form.iter_mut() {
            md.key = utils::replace_variable(md.key.clone(), envs.clone());
            md.value = utils::replace_variable(md.value.clone(), envs.clone());
        }
        for md in build_request.body.body_form_data.iter_mut() {
            md.key = utils::replace_variable(md.key.clone(), envs.clone());
            md.value = utils::replace_variable(md.value.clone(), envs.clone());
        }
        build_request
    }

    fn build_header(
        headers: Vec<Header>,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> Vec<Header> {
        headers
            .iter()
            .filter(|h| h.enable)
            .map(|h| Header {
                key: h.key.clone(),
                value: utils::replace_variable(h.value.clone(), envs.clone()),
                desc: h.desc.clone(),
                enable: h.enable,
                lock_with: h.lock_with.clone(),
            })
            .collect()
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
