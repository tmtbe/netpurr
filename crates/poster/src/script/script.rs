use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;

use deno_core::anyhow::Error;
use deno_core::error::AnyError;
use deno_core::url::Url;
use deno_core::{op2, ExtensionBuilder, Op, OpState};
use deno_core::{ModuleCode, PollEventLoopOptions};
use poll_promise::Promise;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

use crate::data::{
    EnvironmentItemValue, EnvironmentValueType, Header, LockWith, Logger, QueryParam, Request,
};
use crate::script::loader::SimpleModuleLoader;

#[derive(Default)]
pub struct ScriptRuntime {}

#[derive(Clone, Default)]
pub struct Context {
    pub scope_name: String,
    pub request: Request,
    pub envs: BTreeMap<String, EnvironmentItemValue>,
    pub shared_map: BTreeMap<String, String>,
    pub logger: Logger,
}

#[derive(Default, Clone)]
pub struct ScriptScope {
    pub script: String,
    pub scope: String,
}
impl ScriptRuntime {
    pub fn run(
        &self,
        scripts: Vec<ScriptScope>,
        context: Context,
    ) -> Promise<Result<Context, Error>> {
        Promise::spawn_thread("script", || ScriptRuntime::run_block_many(scripts, context))
    }

    pub fn run_block_many(
        scripts: Vec<ScriptScope>,
        mut context: Context,
    ) -> Result<Context, Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        for script_scope in scripts.iter() {
            context.scope_name = script_scope.scope.clone();
            context
                .logger
                .add_info(script_scope.scope.clone(), "execute script.".to_string());
            let step_context = runtime.block_on(async {
                ScriptRuntime::run_js(script_scope.script.clone(), context.clone()).await
            })?;
            context.envs = step_context.envs.clone();
            context.request = step_context.request.clone();
            context.logger = step_context.logger.clone();
            context.shared_map = step_context.shared_map.clone();
        }
        Ok(context)
    }

    async fn run_js(js: String, context: Context) -> Result<Context, Error> {
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
            ])
            .build();
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(SimpleModuleLoader)),
            extensions: vec![runjs_extension],
            ..Default::default()
        });
        js_runtime.op_state().borrow_mut().put(context);
        let runtime_init_code = include_str!("./resource/runtime.js");
        js_runtime
            .execute_script_static("[runjs:runtime.js]", runtime_init_code)
            .unwrap();
        let temp = Url::from_file_path(Path::new("/poster/pre-request-script.js")).unwrap();
        let mod_id = js_runtime
            .load_main_module(&temp, Some(ModuleCode::from(js)))
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
            .ok_or(Error::msg("get context error"))?
            .clone();
        Ok(new_context)
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
struct JsRequest {
    method: String,
    url: String,
    headers: Vec<JsHeader>,
    body: String,
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct JsHeader {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct JsResponse {
    status: u16,
    headers: Vec<JsHeader>,
    text: String,
}

#[op2(async)]
#[serde]
async fn op_http_fetch(#[serde] request: JsRequest) -> Result<JsResponse, AnyError> {
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
