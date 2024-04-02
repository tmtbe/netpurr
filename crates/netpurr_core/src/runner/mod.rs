use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use anyhow::Error;
use async_recursion::async_recursion;
use deno_core::futures::future::join_all;
use deno_core::futures::FutureExt;
use log::info;
use poll_promise::Promise;
use rayon::{Scope, ThreadPool, ThreadPoolBuilder};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use reqwest_cookie_store::CookieStoreMutex;
use rest::RestSender;

use crate::data::collections::{CollectionFolder, CollectionFolderOnlyRead, Testcase};
use crate::data::environment::EnvironmentItemValue;
use crate::data::http::{Request, Response};
use crate::data::logger::Logger;
use crate::data::record::Record;
use crate::data::test::{TestResult, TestStatus};
use crate::data::websocket::WebSocketSession;
use crate::runner;
use crate::runner::websocket::WebSocketSender;
use crate::script::{Context, JsResponse, ScriptRuntime, ScriptScope, ScriptTree, SharedMap, SkipError};

mod rest;
pub mod test;
mod websocket;

#[derive(Clone)]
pub struct Runner {
    script_runtime: ScriptRuntime,
    client: Client,
}
#[derive(Clone, Debug)]
pub struct RunRequestInfo {
    pub shared_map:SharedMap,
    pub collection_path: Option<String>,
    pub request_name: String,
    pub request: Request,
    pub envs: BTreeMap<String, EnvironmentItemValue>,
    pub pre_request_scripts: Vec<ScriptScope>,
    pub test_scripts: Vec<ScriptScope>,
    pub testcase: Testcase,
}
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct TestRunResult {
    pub request: Request,
    pub response: Option<Response>,
    pub test_result: TestResult,
    pub collection_path: Option<String>,
    pub request_name: String,
    pub testcase: Testcase,
}
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct TestRunError {
    pub request: Request,
    pub response: Option<Response>,
    pub collection_path: Option<String>,
    pub request_name: String,
    pub testcase: Testcase,
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
    pub async fn send_rest_with_script_async(
        run_request_info: RunRequestInfo,
        client: Client,
    ) -> Result<TestRunResult, TestRunError> {
        info!("start send_rest_with_script_async:{:?}",run_request_info);
        let shared_map = run_request_info.shared_map;
        let mut logger = Logger::default();
        let mut default_context = Context {
            scope_name: "".to_string(),
            request: run_request_info.request.clone(),
            envs: run_request_info.envs.clone(),
            testcase: run_request_info.testcase.clone(),
            shared_map,
            ..Default::default()
        };
        default_context
            .logger
            .add_info("System".to_string(), format!("Testcase: \n{}",serde_yaml::to_string(&default_context.testcase).unwrap()));
        default_context
            .logger
            .add_info("System".to_string(), format!("Envs: \n{}",serde_yaml::to_string(&default_context.envs).unwrap()));
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
                logger.add_info("System".to_string(),format!("Envs: \n{}",serde_yaml::to_string(&pre_request_context.envs).unwrap()));
                let build_request = RestSender::build_request(
                    pre_request_context.request.clone(),
                    pre_request_context.envs.clone(),
                );
                logger.add_info(
                    "Fetch".to_string(),
                    format!("start fetch request: \n{}", serde_yaml::to_string(&build_request).unwrap()),
                );
                match RestSender::reqwest_async_send(build_request, client).await {
                    Ok((after_request, response)) => {
                        let mut after_response = response;
                        logger.add_info(
                            "Fetch".to_string(),
                            "get response".to_string(),
                        );
                        after_response.logger = logger;
                        let mut test_result: TestResult = Default::default();
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
                                    if e.to_string().contains("TestSkip"){
                                        test_result.status = TestStatus::SKIP;
                                        for test_info in test_result.test_info_list.iter_mut() {
                                            test_info.status = TestStatus::SKIP;
                                        }
                                    } else {
                                        return Err(TestRunError {
                                            request: after_request,
                                            response: Some(after_response),
                                            collection_path: run_request_info.collection_path.clone(),
                                            request_name: run_request_info.request_name,
                                            testcase: run_request_info.testcase.clone(),
                                            error: e.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        Ok(TestRunResult {
                            request: after_request,
                            response: Some(after_response),
                            test_result,
                            collection_path: run_request_info.collection_path.clone(),
                            request_name: run_request_info.request_name.clone(),
                            testcase: run_request_info.testcase.clone(),
                        })
                    }
                    Err(e) => Err(TestRunError {
                        request: run_request_info.request.clone(),
                        response: None,
                        collection_path: run_request_info.collection_path.clone(),
                        request_name: run_request_info.request_name,
                        testcase: run_request_info.testcase.clone(),
                        error: e.to_string(),
                    }),
                }
            }
            Err(e) => {
                if e.to_string().contains("TestSkip"){
                    let mut test_result = TestResult::default();
                    test_result.status = TestStatus::SKIP;
                    Ok(TestRunResult {
                        request: run_request_info.request.clone(),
                        response: None,
                        test_result,
                        collection_path: run_request_info.collection_path.clone(),
                        request_name: run_request_info.request_name.clone(),
                        testcase: run_request_info.testcase.clone(),
                    })
                } else {
                    Err(TestRunError {
                        request: run_request_info.request.clone(),
                        response: None,
                        collection_path: run_request_info.collection_path.clone(),
                        request_name: run_request_info.request_name,
                        testcase: run_request_info.testcase.clone(),
                        error: e.to_string(),
                    })
                }
            },
        }
    }
    pub fn send_rest_with_script_promise(
        &self,
        mut run_request_info: RunRequestInfo,
    ) -> Promise<Result<TestRunResult, TestRunError>> {
        let client = self.client.clone();
        run_request_info.shared_map = SharedMap::default();
        Promise::spawn_thread("send_with_script", move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(Self::send_rest_with_script_async(
                run_request_info,
                client
            ))
        })
    }

    pub fn run_test_group_promise(
        &self,
        fast:bool,
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) -> Promise<()> {
        let client = self.client.clone();
        let folder_only_read = CollectionFolderOnlyRead::from(folder);
        let run_request_infos = Self::get_test_group_jobs(envs,script_tree,collection_path,parent_testcase,folder_only_read);
        Promise::spawn_thread("send_with_script", move || {
            Self::run_test_group_jobs(client,run_request_infos,test_group_run_result.clone(),fast);
        })
    }
    pub fn run_test_record_promise(
        &self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        record: Record,
    ) -> Promise<()> {
        let client = self.client.clone();
        Promise::spawn_thread("send_with_script", move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async {
                Self::run_test_record_async(
                    client.clone(),
                    envs.clone(),
                    script_tree.clone(),
                    parent_testcase,
                    test_group_run_result.clone(),
                    collection_path.clone(),
                    record.clone(),
                )
                .await
            })
        })
    }

    async fn run_test_record_async(
        client: Client,
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        testcase: Option<Testcase>,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_path: String,
        record: Record,
    ) {
        let mut run_request_infos = vec![];
        // 每个文件夹的shared_map是隔离的
        let shared_map = SharedMap::default();
        let mut record_testcases = record.testcase().clone();
        if record_testcases.is_empty() {
            let mut testcase = Testcase::default();
            record_testcases.insert(testcase.name.clone(), testcase);
        }
        for (_, request_testcase) in record_testcases.iter() {
            let mut record_pre_request_parent_script_scopes = script_tree
                .get_pre_request_parent_script_scope(collection_path.clone())
                .clone();
            let scope = format!("{}/{}", collection_path.clone(), record.name());
            if record.pre_request_script() != "" {
                record_pre_request_parent_script_scopes.push(ScriptScope {
                    scope: scope.clone(),
                    script: record.pre_request_script(),
                });
            }
            let mut record_test_parent_script_scopes = script_tree
                .get_test_parent_script_scope(collection_path.clone())
                .clone();
            if record.test_script() != "" {
                record_test_parent_script_scopes.push(ScriptScope {
                    scope: scope.clone(),
                    script: record.test_script(),
                });
            }
            let mut new_request_testcase = request_testcase.clone();
            testcase.clone().map(|t| {
                new_request_testcase.merge(record.name(), &t);
            });
            let run_request_info = RunRequestInfo {
                shared_map:shared_map.clone(),
                collection_path: Some(collection_path.clone()),
                request_name: record.name(),
                request: record.must_get_rest().request.clone(),
                envs: envs.clone(),
                pre_request_scripts: record_pre_request_parent_script_scopes,
                test_scripts: record_test_parent_script_scopes,
                testcase: new_request_testcase.clone(),
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
                _client
            ));
        }
        let results = join_all(jobs).await;
        test_group_run_result.write().unwrap().add_results(results);
    }
    pub fn run_test_group_jobs(client: Client,run_request_infos:Vec<RunRequestInfo>, test_group_run_result: Arc<RwLock<TestGroupRunResults>>,fast:bool){
        let pool = ThreadPoolBuilder::new().num_threads(20).build().unwrap();
        if fast {
            pool.scope(|scope| {
                for run_request_info in run_request_infos {
                    let _client = client.clone();
                    let _test_group_run_result = test_group_run_result.clone();
                    Self::run_one_job(_client, _test_group_run_result, scope, run_request_info.clone());
                }
            });
        }else{
            let mut groups:HashMap<String,Vec<RunRequestInfo>> = HashMap::new();
            run_request_infos.iter().for_each(|r|{
                let key = r.testcase.parent_path.join("/");
                if !groups.contains_key(key.as_str()){
                    groups.insert(key.clone(),vec![]);
                }
                groups.get_mut(key.as_str()).unwrap().push(r.clone());
            });
            groups.iter().for_each(|(_,rs)|{
                pool.scope(|scope| {
                    for run_request_info in rs {
                        let _client = client.clone();
                        let _test_group_run_result = test_group_run_result.clone();
                        Self::run_one_job(_client, _test_group_run_result, scope, run_request_info.clone());
                    }
                });
            })
        }
        info!("all test_jobs finish");
    }

    fn run_one_job(client: Client, test_group_run_result: Arc<RwLock<TestGroupRunResults>>, scope: &Scope, run_request_info: RunRequestInfo) {
        scope.spawn(move |_| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let mut test_result =TestResult::default();
            test_result.status = TestStatus::RUNNING;
            test_group_run_result.write().unwrap().add_result(Ok(TestRunResult{
                request: run_request_info.request.clone(),
                response: None,
                test_result,
                collection_path: run_request_info.collection_path.clone(),
                request_name: run_request_info.request_name.clone(),
                testcase: run_request_info.testcase.clone(),
            }));
            runtime.block_on(async {
                let result = Self::send_rest_with_script_async(
                    run_request_info.clone(),
                    client
                ).await;
                info!("job finish:{:?}",result);
                test_group_run_result.write().unwrap().add_result(result);
            });
        })
    }
    pub fn get_test_group_jobs(
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        folder: CollectionFolderOnlyRead,
    )->Vec<RunRequestInfo>{
        let mut run_request_infos =vec![];
        let mut testcases = folder.testcases.clone();
        if testcases.is_empty() {
            let testcase = Testcase::default();
            testcases.insert(testcase.name.clone(), testcase);
        }
        for (_, testcase) in testcases.iter() {
            let mut root_testcase = testcase.clone();
            if let Some(parent) = &parent_testcase {
                root_testcase.merge(folder.name.clone(), parent);
            } else {
                root_testcase.entry_name = folder.name.clone();
            }
            let mut result = Self::_get_test_group_jobs(
                envs.clone(),
                script_tree.clone(),
                root_testcase,
                collection_path.clone(),
                folder.clone(),
            );
            run_request_infos.append(&mut result);
        }
        return run_request_infos;
    }
    fn _get_test_group_jobs(
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        testcase: Testcase,
        collection_path: String,
        folder: CollectionFolderOnlyRead,
    )->Vec<RunRequestInfo> {
        let mut run_request_infos = vec![];
        // 每个文件夹的shared_map是隔离的
        let shared_map = SharedMap::default();
        for (name, child_folder) in folder.folders.iter() {
            let mut child_testcases = child_folder.testcases.clone();
            if child_testcases.is_empty() {
                let mut testcase = Testcase::default();
                child_testcases.insert(testcase.name.clone(), testcase);
            }
            for (name, child_testcase) in child_testcases.iter() {
                let mut merge_testcase = child_testcase.clone();
                merge_testcase.merge(child_folder.name.clone(), &testcase);
                let mut result = Self::_get_test_group_jobs(
                    envs.clone(),
                    script_tree.clone(),
                    merge_testcase,
                    collection_path.clone() + "/" + name,
                    child_folder.clone(),
                );
                run_request_infos.append(&mut result);
            }
        }
        for (name, record) in folder.requests.iter() {
            let mut record_testcases = record.testcase().clone();
            if record_testcases.is_empty() {
                let mut testcase = Testcase::default();
                record_testcases.insert(testcase.name.clone(), testcase);
            }
            for (_, request_testcase) in record_testcases.iter() {
                let mut record_pre_request_parent_script_scopes = script_tree
                    .get_pre_request_parent_script_scope(folder.get_path())
                    .clone();
                let scope = format!("{}/{}", collection_path.clone(), name);
                if record.pre_request_script() != "" {
                    record_pre_request_parent_script_scopes.push(ScriptScope {
                        scope: scope.clone(),
                        script: record.pre_request_script(),
                    });
                }
                let mut record_test_parent_script_scopes = script_tree
                    .get_test_parent_script_scope(folder.get_path())
                    .clone();
                if record.test_script() != "" {
                    record_test_parent_script_scopes.push(ScriptScope {
                        scope: scope.clone(),
                        script: record.test_script(),
                    });
                }
                let mut new_request_testcase = request_testcase.clone();
                new_request_testcase.merge(record.name(), &testcase);
                let run_request_info = RunRequestInfo {
                    shared_map:shared_map.clone(),
                    collection_path: Some(collection_path.clone()),
                    request_name: record.name(),
                    request: record.must_get_rest().request.clone(),
                    envs: envs.clone(),
                    pre_request_scripts: record_pre_request_parent_script_scopes,
                    test_scripts: record_test_parent_script_scopes,
                    testcase: new_request_testcase.clone(),
                };
                run_request_infos.push(run_request_info)
            }
        }
        return run_request_infos
    }
}
#[derive(Default, Clone)]
pub struct TestGroupRunResults {
    pub results: HashMap<String, Result<TestRunResult, TestRunError>>,
}

impl TestGroupRunResults {
    pub fn add_result(&mut self, result: Result<TestRunResult, TestRunError>) {
        match &result {
            Ok(r) => {
                let key = r.testcase.get_testcase_path().join("/");
                self.results.insert(key, result.clone());
            }
            Err(e) => {
                let key = e.testcase.get_testcase_path().join("/");
                self.results.insert(key, result.clone());
            }
        };
    }
    pub fn add_results(&mut self, results: Vec<Result<TestRunResult, TestRunError>>) {
        for result in results.iter() {
            self.add_result(result.clone());
        }
    }

    pub fn find(&self, testcase_path: Vec<String>) -> Option<Result<TestRunResult, TestRunError>> {
        let key = testcase_path.join("/");
        return self.results.get(key.as_str()).cloned();
    }
}
