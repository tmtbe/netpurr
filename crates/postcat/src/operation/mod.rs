use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use eframe::emath::Align2;
use egui_toast::Toasts;
use poll_promise::Promise;
use reqwest::blocking::Client;

use rest_sender::RestSender;
use windows::OpenWindows;

use crate::data::environment::EnvironmentItemValue;
use crate::data::logger::Logger;
use crate::data::{http, test};
use crate::operation::windows::Windows;
use crate::script::script::{Context, JsResponse, ScriptRuntime, ScriptScope};

pub mod rest_sender;
pub mod windows;

pub struct Operation {
    rest_sender: RestSender,
    open_windows: OpenWindows,
    lock_ui: HashMap<String, bool>,
    script_runtime: ScriptRuntime,
    toasts: Toasts,
    windows: Windows,
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
            windows: Windows::default(),
        }
    }
}

impl Operation {
    pub fn send_with_script(
        &self,
        request: http::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        pre_request_scripts: Vec<ScriptScope>,
        test_scripts: Vec<ScriptScope>,
        client: Client,
    ) -> Promise<Result<(http::Request, http::Response, test::TestResult), String>> {
        let mut logger = Logger::default();
        Promise::spawn_thread("send_with_script", move || {
            let mut pre_request_context_result = Ok(Context {
                scope_name: "".to_string(),
                request: request.clone(),
                envs: envs.clone(),
                ..Default::default()
            });
            if pre_request_scripts.len() > 0 {
                pre_request_context_result = ScriptRuntime::run_block_many(
                    pre_request_scripts,
                    Context {
                        scope_name: "".to_string(),
                        request: request.clone(),
                        envs: envs.clone(),
                        ..Default::default()
                    },
                );
            }
            match pre_request_context_result {
                Ok(pre_request_context) => {
                    for log in pre_request_context.logger.logs.iter() {
                        logger.logs.push(log.clone());
                    }
                    let build_request = RestSender::build_request(
                        pre_request_context.request.clone(),
                        pre_request_context.envs.clone(),
                    );
                    logger.add_info(
                        "fetch".to_string(),
                        format!("start fetch request: {:?}", build_request),
                    );
                    match RestSender::reqwest_block_send(build_request, client) {
                        Ok((after_request, response)) => {
                            let mut after_response = response;
                            logger.add_info(
                                "fetch".to_string(),
                                format!("get response: {:?}", after_response),
                            );
                            after_response.logger = logger;
                            let mut test_result: test::TestResult = Default::default();
                            let mut test_context = pre_request_context.clone();
                            test_context.response =
                                JsResponse::from_data_response(after_response.clone());
                            if test_scripts.len() > 0 {
                                pre_request_context_result =
                                    ScriptRuntime::run_block_many(test_scripts, test_context);
                                match pre_request_context_result {
                                    Ok(test_context) => {
                                        for log in test_context.logger.logs.iter() {
                                            after_response.logger.logs.push(log.clone());
                                        }
                                        test_result = test_context.test_result.clone();
                                    }
                                    Err(e) => {
                                        return Err(e.to_string());
                                    }
                                }
                            }
                            Ok((after_request, after_response, test_result))
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

    pub fn windows(&mut self) -> &mut Windows {
        &mut self.windows
    }
}
