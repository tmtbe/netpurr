use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_recursion::async_recursion;
use deno_core::futures::future::join_all;
use deno_core::futures::FutureExt;
use poll_promise::Promise;
use reqwest::Client;

use reqwest_cookie_store::CookieStoreMutex;
use rest::RestSender;

use crate::data::collections::{CollectionFolder, CollectionFolderOnlyRead};
use crate::data::environment::EnvironmentItemValue;
use crate::data::http::{Request, Response};
use crate::data::logger::Logger;
use crate::data::test;
use crate::data::test::TestResult;
use crate::data::websocket::WebSocketSession;
use crate::runner::websocket::WebSocketSender;
use crate::script::{Context, JsResponse, ScriptRuntime, ScriptScope, SharedMap};

mod rest;
mod websocket;

#[derive(Clone)]
pub struct Runner {
    script_runtime: ScriptRuntime,
    client: Client,
}
#[derive(Clone)]
pub struct RunRequestInfo {
    pub collection_path: Option<String>,
    pub request_name: String,
    pub request: Request,
    pub envs: BTreeMap<String, EnvironmentItemValue>,
    pub pre_request_scripts: Vec<ScriptScope>,
    pub test_scripts: Vec<ScriptScope>,
}
#[derive(Default, Clone, Debug)]
pub struct TestRunResult {
    pub request: Request,
    pub response: Response,
    pub test_result: TestResult,
    pub collection_path: Option<String>,
    pub request_name: String,
}
#[derive(Default, Clone, Debug)]
pub struct TestRunError {
    pub collection_path: Option<String>,
    pub request_name: String,
    pub error: String,
}
impl Runner {
    pub fn new(cookie_store: Arc<CookieStoreMutex>) -> Self {
        Runner {
            script_runtime: Default::default(),
            client: Client::builder()
                .cookie_provider(cookie_store)
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
        self.script_runtime.run_block(scripts, context)
    }

    pub fn connect_websocket_with_script(
        &self,
        run_request_info: RunRequestInfo,
    ) -> WebSocketSession {
        WebSocketSender::connect(run_request_info.request)
    }
    async fn send_rest_with_script_async(
        run_request_info: RunRequestInfo,
        client: Client,
        shared_map: SharedMap,
    ) -> Result<TestRunResult, TestRunError> {
        let mut logger = Logger::default();
        let mut default_context = Context {
            scope_name: "".to_string(),
            request: run_request_info.request.clone(),
            envs: run_request_info.envs.clone(),
            shared_map,
            ..Default::default()
        };
        let mut pre_request_context_result = Ok(default_context.clone());
        if run_request_info.pre_request_scripts.len() > 0 {
            default_context
                .logger
                .add_info("System".to_string(), "Run pre-request-scripts".to_string());
            pre_request_context_result =
                ScriptRuntime::run_async(run_request_info.pre_request_scripts, default_context)
                    .await;
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
                match RestSender::reqwest_async_send(build_request, client).await {
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
                        if run_request_info.test_scripts.len() > 0 {
                            after_response
                                .logger
                                .add_info("System".to_string(), "Run Test-script".to_string());
                            pre_request_context_result = ScriptRuntime::run_async(
                                run_request_info.test_scripts,
                                test_context,
                            )
                            .await;
                            match pre_request_context_result {
                                Ok(test_context) => {
                                    for log in test_context.logger.logs.iter() {
                                        after_response.logger.logs.push(log.clone());
                                    }
                                    test_result = test_context.test_result.clone();
                                }
                                Err(e) => {
                                    return Err(TestRunError {
                                        collection_path: run_request_info.collection_path.clone(),
                                        request_name: run_request_info.request_name,
                                        error: e.to_string(),
                                    });
                                }
                            }
                        }
                        Ok(TestRunResult {
                            request: after_request,
                            response: after_response,
                            test_result,
                            collection_path: run_request_info.collection_path.clone(),
                            request_name: run_request_info.request_name.clone(),
                        })
                    }
                    Err(e) => Err(TestRunError {
                        collection_path: run_request_info.collection_path.clone(),
                        request_name: run_request_info.request_name,
                        error: e.to_string(),
                    }),
                }
            }
            Err(e) => Err(TestRunError {
                collection_path: run_request_info.collection_path.clone(),
                request_name: run_request_info.request_name,
                error: e.to_string(),
            }),
        }
    }
    pub fn send_rest_with_script_promise(
        &self,
        run_request_info: RunRequestInfo,
    ) -> Promise<Result<TestRunResult, TestRunError>> {
        let client = self.client.clone();
        let shared_map = SharedMap::default();
        Promise::spawn_thread("send_with_script", move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(Self::send_rest_with_script_async(
                run_request_info,
                client,
                shared_map,
            ))
        })
    }

    pub fn run_test_group_promise(
        &self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        pre_request_parent_script_scopes: Vec<ScriptScope>,
        test_parent_script_scopes: Vec<ScriptScope>,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_name: String,
        collection_path: String,
        folder: Rc<RefCell<CollectionFolder>>,
    ) -> Promise<()> {
        let client = self.client.clone();
        let folder_only_read = CollectionFolderOnlyRead::from(folder);
        Promise::spawn_thread("send_with_script", move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async {
                Self::run_test_group_async(
                    client,
                    envs,
                    pre_request_parent_script_scopes,
                    test_parent_script_scopes,
                    test_group_run_result,
                    collection_name,
                    collection_path,
                    folder_only_read,
                )
                .await
            })
        })
    }

    #[async_recursion(?Send)]
    async fn run_test_group_async(
        client: Client,
        envs: BTreeMap<String, EnvironmentItemValue>,
        pre_request_parent_script_scopes: Vec<ScriptScope>,
        test_parent_script_scopes: Vec<ScriptScope>,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_name: String,
        collection_path: String,
        folder: CollectionFolderOnlyRead,
    ) {
        let mut run_request_infos = vec![];
        // 每个文件夹的shared_map是隔离的
        let shared_map = SharedMap::default();
        for (name, folder) in folder.folders.iter() {
            Self::run_test_group_async(
                client.clone(),
                envs.clone(),
                pre_request_parent_script_scopes.clone(),
                test_parent_script_scopes.clone(),
                test_group_run_result.clone(),
                collection_name.clone(),
                collection_path.clone() + "/" + name,
                folder.clone(),
            )
            .await;
        }
        for (name, record) in folder.requests.iter() {
            let mut record_pre_request_parent_script_scopes =
                pre_request_parent_script_scopes.clone();
            let scope = format!("{}/{}", collection_path.clone(), name);
            if record.pre_request_script() != "" {
                record_pre_request_parent_script_scopes.push(ScriptScope {
                    scope: scope.clone(),
                    script: record.pre_request_script(),
                });
            }
            let mut record_test_parent_script_scopes = test_parent_script_scopes.clone();
            if record.test_script() != "" {
                record_test_parent_script_scopes.push(ScriptScope {
                    scope: scope.clone(),
                    script: record.test_script(),
                });
            }
            let run_request_info = RunRequestInfo {
                collection_path: Some(collection_path.clone()),
                request_name: record.name(),
                request: record.must_get_rest().request.clone(),
                envs: envs.clone(),
                pre_request_scripts: record_pre_request_parent_script_scopes,
                test_scripts: record_test_parent_script_scopes,
            };
            run_request_infos.push(run_request_info)
        }
        let mut jobs = vec![];
        for run_request_info in run_request_infos.iter() {
            let _client = client.clone();
            let _run_request_info = run_request_info.clone();
            let _shared_map = shared_map.clone();
            jobs.push(Self::send_rest_with_script_async(
                _run_request_info,
                _client,
                _shared_map,
            ));
        }
        let results = join_all(jobs).await;
        test_group_run_result.write().unwrap().add_results(results);
    }
}
#[derive(Default, Clone)]
pub struct TestGroupRunResults {
    pub results: HashMap<String, Result<TestRunResult, TestRunError>>,
}

impl TestGroupRunResults {
    pub fn add_result(&mut self, result: Result<TestRunResult, TestRunError>) {
        match &result {
            Ok(r) => self.results.insert(
                format!(
                    "{}/{}",
                    r.collection_path.clone().unwrap_or_default(),
                    r.request_name
                ),
                result.clone(),
            ),
            Err(e) => self.results.insert(
                format!(
                    "{}/{}",
                    e.collection_path.clone().unwrap_or_default(),
                    e.request_name
                ),
                result.clone(),
            ),
        };
    }
    pub fn add_results(&mut self, results: Vec<Result<TestRunResult, TestRunError>>) {
        for result in results.iter() {
            self.add_result(result.clone());
        }
    }

    pub fn find(&self, path: String, name: String) -> Option<Result<TestRunResult, TestRunError>> {
        let key = format!("{}/{}", path, name);
        return self.results.get(key.as_str()).cloned();
    }
}
