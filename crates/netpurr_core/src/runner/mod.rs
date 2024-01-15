use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use poll_promise::Promise;
use reqwest::blocking::Client;

use rest::RestSender;

use crate::data::cookies_manager::CookiesManager;
use crate::data::environment::EnvironmentItemValue;
use crate::data::logger::Logger;
use crate::data::{http, test};
use crate::script::{Context, JsResponse, ScriptRuntime, ScriptScope};

mod rest;

#[derive(Clone)]
pub struct Runner {
    script_runtime: ScriptRuntime,
    client: Client,
}

impl Runner {
    pub fn new(cookies_manager: CookiesManager) -> Self {
        Runner {
            script_runtime: Default::default(),
            client: Client::builder()
                .cookie_provider(cookies_manager.cookie_store.clone())
                .trust_dns(true)
                .tcp_nodelay(true)
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }
    pub fn run_script(
        &self,
        scripts: Vec<ScriptScope>,
        context: Context,
    ) -> Promise<anyhow::Result<Context>> {
        self.script_runtime.run(scripts, context)
    }
    pub fn send_with_script(
        &self,
        request: http::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        pre_request_scripts: Vec<ScriptScope>,
        test_scripts: Vec<ScriptScope>,
    ) -> Promise<Result<(http::Request, http::Response, test::TestResult), String>> {
        let mut logger = Logger::default();
        let client = self.client.clone();
        Promise::spawn_thread("send_with_script", move || {
            let mut default_context = Context {
                scope_name: "".to_string(),
                request: request.clone(),
                envs: envs.clone(),
                ..Default::default()
            };
            let mut pre_request_context_result = Ok(default_context.clone());
            if pre_request_scripts.len() > 0 {
                default_context
                    .logger
                    .add_info("System".to_string(), "Run pre-request-scripts".to_string());
                pre_request_context_result =
                    ScriptRuntime::run_block_many(pre_request_scripts, default_context);
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
                            test_context.logger = Logger::default();
                            if test_scripts.len() > 0 {
                                after_response
                                    .logger
                                    .add_info("System".to_string(), "Run Test-script".to_string());
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
}
