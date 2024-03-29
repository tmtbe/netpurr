use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Error;
use deno_core::{ExtensionBuilder, FsModuleLoader, ModuleCodeString, Op, op2, OpState};
use deno_core::{JsRuntime, PollEventLoopOptions};
use deno_core::url::Url;
use poll_promise::Promise;
use reqwest::{Client, Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::data::collections::Testcase;
use crate::data::environment::{EnvironmentItemValue, EnvironmentValueType};
use crate::data::http;
use crate::data::http::{Header, LockWith, QueryParam, Request};
use crate::data::logger::Logger;
use crate::data::test::TestResult;

#[derive(Default, Clone)]
pub struct ScriptRuntime {}

#[derive(Clone, Default)]
pub struct Context {
    pub scope_name: String,
    pub request: Request,
    pub response: JsResponse,
    pub envs: BTreeMap<String, EnvironmentItemValue>,
    pub testcase: Testcase,
    pub shared_map: SharedMap,
    pub logger: Logger,
    pub test_result: TestResult,
}

#[derive(Default, Clone)]
pub struct SharedMap {
    map: Arc<RwLock<BTreeMap<String, String>>>,
}
impl Deref for SharedMap {
    type Target = Arc<RwLock<BTreeMap<String, String>>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[derive(Default, Clone,Debug)]
pub struct ScriptScope {
    pub script: String,
    pub scope: String,
}
#[derive(Default, Clone)]
pub struct ScriptTree {
    pub pre_request_parent_script_scopes: BTreeMap<String, Vec<ScriptScope>>,
    pub test_parent_script_scopes: BTreeMap<String, Vec<ScriptScope>>,
}
impl ScriptRuntime {
    pub fn run_block(
        &self,
        scripts: Vec<ScriptScope>,
        context: Context,
    ) -> Promise<anyhow::Result<Context>> {
        Promise::spawn_thread("script", || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(ScriptRuntime::run_async(scripts, context))
        })
    }

    pub async fn run_async(
        scripts: Vec<ScriptScope>,
        mut context: Context,
    ) -> anyhow::Result<Context> {
        for script_scope in scripts.iter() {
            context.scope_name = script_scope.scope.clone();
            let step_context =
                ScriptRuntime::run_js(script_scope.script.clone(), context.clone()).await?;
            context.envs = step_context.envs.clone();
            context.request = step_context.request.clone();
            context.logger = step_context.logger.clone();
            context.shared_map = step_context.shared_map.clone();
            context.test_result = step_context.test_result.clone();
        }
        Ok(context)
    }

    fn build_js_runtime() -> JsRuntime {
        let runjs_extension = ExtensionBuilder::default()
            .ops(vec![
                op_set_env::DECL,
                op_get_env::DECL,
                op_add_params::DECL,
                op_add_header::DECL,
                op_log::DECL,
                op_error::DECL,
                op_warn::DECL,
                op_http_fetch::DECL,
                op_get_shared::DECL,
                op_set_shared::DECL,
                op_wait_shared::DECL,
                op_response::DECL,
                op_open_test::DECL,
                op_close_test::DECL,
                op_append_assert::DECL,
                op_sleep::DECL,
                op_get_testcase::DECL,
                op_test_skip::DECL,
                op_equal::DECL
            ])
            .build();
        return JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(FsModuleLoader)),
            extensions: vec![runjs_extension],
            ..Default::default()
        });
    }
    async fn run_js(js: String, context: Context) -> anyhow::Result<Context> {
        let mut js_runtime = Self::build_js_runtime();
        js_runtime.op_state().borrow_mut().put(context);
        let runtime_init_code = include_str!("resource/runtime.js");
        js_runtime
            .execute_script_static("[runjs:runtime.js]", runtime_init_code)
            .unwrap();
        let temp = Url::from_file_path(Path::new("/netpurr/pre-request-script.js")).unwrap();
        let mod_id = js_runtime
            .load_main_module(&temp, Some(ModuleCodeString::from(js)))
            .await?;
        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await?;
        result.await?;
        let op_state = js_runtime.op_state();
        let new_context = op_state
            .borrow()
            .try_borrow::<Context>()
            .ok_or(anyhow::Error::msg("get context error"))?
            .clone();
        Ok(new_context)
    }
}

#[op2(fast)]
fn op_set_shared(state: &mut OpState, #[string] key: String, #[string] value: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => {
            c.shared_map
                .write()
                .unwrap()
                .insert(key.clone(), value.clone());
            c.logger.add_info(
                c.scope_name.clone(),
                format!("set shared: `{}` as `{}`", key, value),
            );
        }
    }
}
#[op2(async)]
async fn op_sleep(#[bigint] time: u64) -> anyhow::Result<()> {
    sleep(Duration::from_millis(time)).await;
    Ok(())
}
#[op2(async)]
#[string]
async fn op_wait_shared(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> anyhow::Result<String> {
    let mut _state = state.borrow_mut();
    let mut count = 0;
    let context = _state.try_borrow_mut::<Context>();
    match context {
        None => {
            return Err(Error::msg("context is none"));
        }
        Some(c) => loop {
            let value = c.shared_map.read().unwrap().get(key.as_str()).cloned();
            match value {
                None => {
                    sleep(Duration::from_millis(100)).await;
                    count = count + 1;
                    if count > 100 {
                        c.logger.add_error(
                            c.scope_name.clone(),
                            format!("get shared `{}` failed", key),
                        );
                        return Err(Error::msg(format!("get shared value:{} time out", key)));
                    }
                }
                Some(v) => {
                    c.logger.add_info(
                        c.scope_name.clone(),
                        format!("get shared: `{}` as `{}`", key, v),
                    );
                    return Ok(v.clone());
                }
            }
        },
    }
}

#[op2]
#[string]
fn op_get_shared(state: &mut OpState, #[string] key: String) -> String {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => "".to_string(),
        Some(c) => match c.shared_map.read().unwrap().get(key.as_str()).cloned() {
            None => {
                c.logger
                    .add_error(c.scope_name.clone(), format!("get shared `{}` failed", key));
                "\"\"".to_string()
            }
            Some(v) => {
                c.logger.add_info(
                    c.scope_name.clone(),
                    format!("get shared: `{}` as `{}`", key, v),
                );
                v.clone()
            }
        },
    }
}

#[op2]
#[string]
fn op_get_testcase(state: &mut OpState) -> anyhow::Result<String> {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => Err(Error::msg("context is none")),
        Some(c) => {
            let json = serde_json::to_string(&c.testcase.value).unwrap();
            Ok(json)
        }
    }
}

#[op2(fast)]
fn op_set_env(state: &mut OpState, #[string] key: String, #[string] value: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => {
            c.envs.insert(
                key.clone(),
                EnvironmentItemValue {
                    value: value.clone(),
                    scope: "Script".to_string(),
                    value_type: EnvironmentValueType::String,
                },
            );
            c.logger.add_info(
                c.scope_name.clone(),
                format!("set env: `{}` as `{}`", key, value),
            );
        }
    }
}

#[op2]
#[string]
fn op_get_env(state: &mut OpState, #[string] key: String) -> String {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => "".to_string(),
        Some(c) => match c.envs.get(key.as_str()).cloned() {
            None => {
                c.logger
                    .add_error(c.scope_name.clone(), format!("get env `{}` failed", key));
                "".to_string()
            }
            Some(v) => v.value.clone(),
        },
    }
}

#[op2(fast)]
fn op_add_header(state: &mut OpState, #[string] key: String, #[string] value: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => {
            c.request.headers.push(Header {
                key: key.clone(),
                value: value.clone(),
                enable: true,
                lock_with: LockWith::LockWithScript,
                desc: "build with script".to_string(),
                ..Default::default()
            });
            c.logger.add_info(
                c.scope_name.clone(),
                format!("add header: `{}` as `{}`", key, value),
            );
        }
    }
}

#[op2(fast)]
fn op_add_params(state: &mut OpState, #[string] key: String, #[string] value: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => {
            c.request.params.push(QueryParam {
                key: key.clone(),
                value: value.clone(),
                enable: true,
                lock_with: LockWith::LockWithScript,
                desc: "build with script".to_string(),
                ..Default::default()
            });
            c.logger.add_info(
                c.scope_name.clone(),
                format!("add params: `{}` as `{}`", key, value),
            );
        }
    }
}

#[op2(fast)]
fn op_log(state: &mut OpState, #[string] msg: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.logger.add_info(c.scope_name.clone(), msg),
    }
}

#[op2(fast)]
fn op_error(state: &mut OpState, #[string] msg: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.logger.add_error(c.scope_name.clone(), msg),
    }
}

#[op2(fast)]
fn op_warn(state: &mut OpState, #[string] msg: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.logger.add_warn(c.scope_name.clone(), msg),
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct JsRequest {
    method: String,
    url: String,
    headers: Vec<JsHeader>,
    body: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct JsHeader {
    name: String,
    value: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct JsResponse {
    status: u16,
    headers: Vec<JsHeader>,
    text: String,
}

impl JsResponse {
    pub fn from_data_response(response: http::Response) -> Self {
        Self {
            status: response.status,
            headers: response
                .headers
                .iter()
                .map(|h| JsHeader {
                    name: h.key.clone(),
                    value: h.value.clone(),
                })
                .collect(),
            text: String::from_utf8(response.body.to_vec()).unwrap_or("".to_string()),
        }
    }
}
#[op2(async)]
#[serde]
async fn op_http_fetch(#[serde] request: JsRequest) -> anyhow::Result<JsResponse> {
    let method_enum = Method::from_str(request.method.to_uppercase().as_str())?;
    let mut request_headers = HeaderMap::new();
    for header in request.headers.iter() {
        request_headers.insert(
            HeaderName::from_str(header.value.as_str())?,
            HeaderValue::from_str(header.value.as_str())?,
        );
    }
    let response = Client::builder()
        .build()?
        .request(method_enum, request.url)
        .headers(request_headers)
        .body(request.body)
        .send()
        .await?;
    let status = response.status().as_u16();
    let mut response_headers: Vec<JsHeader> = vec![];
    for (header_name, header_value) in response.headers().iter() {
        response_headers.push(JsHeader {
            name: header_name.to_string(),
            value: header_value.to_str()?.to_string(),
        });
    }
    let text = response.text().await?.clone();
    let result = JsResponse {
        status,
        text,
        headers: response_headers,
    };
    Ok(result)
}

#[op2]
#[serde]
fn op_response(state: &mut OpState) -> JsResponse {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => JsResponse::default(),
        Some(c) => c.response.clone(),
    }
}

#[op2(fast)]
fn op_open_test(state: &mut OpState, #[string] test_name: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.test_result.open(test_name),
    }
}

#[op2(fast)]
fn op_close_test(state: &mut OpState, #[string] test_name: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.test_result.close(test_name),
    }
}

#[op2(fast)]
fn op_append_assert(state: &mut OpState, result: bool, #[string] msg: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.test_result.append(result, msg),
    }
}

#[op2]
fn op_equal(#[serde] a:serde_json::Value, #[serde] b:serde_json::Value) -> bool {
    return a==b;
}

#[op2(fast)]
fn op_test_skip()-> anyhow::Result<()> {
    Err(Error::from(SkipError {}))
}

#[derive(Debug)]
pub struct SkipError {
}

impl Display for SkipError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}","TestSkip")
    }
}

impl error::Error for SkipError {}


impl ScriptTree {
    pub fn add_pre_request_parent_script_scope(&mut self, path: String, scopes: Vec<ScriptScope>) {
        self.pre_request_parent_script_scopes.insert(path, scopes);
    }
    pub fn add_test_parent_script_scope(&mut self, path: String, scopes: Vec<ScriptScope>) {
        self.test_parent_script_scopes.insert(path, scopes);
    }
    pub fn get_pre_request_parent_script_scope(&self, path: String) -> Vec<ScriptScope> {
        self.pre_request_parent_script_scopes
            .get(path.as_str())
            .cloned()
            .unwrap_or_default()
    }
    pub fn get_test_parent_script_scope(&self, path: String) -> Vec<ScriptScope> {
        self.test_parent_script_scopes
            .get(path.as_str())
            .cloned()
            .unwrap_or_default()
    }
}
