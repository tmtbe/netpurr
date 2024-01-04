use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

use deno_core::anyhow::Error;
use deno_core::url::Url;
use deno_core::{op2, ExtensionBuilder, Op, OpState};
use deno_core::{ModuleCode, PollEventLoopOptions};
use poll_promise::Promise;

use crate::data::{
    EnvironmentItemValue, EnvironmentValueType, Header, LockWith, QueryParam, Request,
};

#[derive(Default)]
pub struct ScriptRuntime {}

#[derive(Clone)]
pub struct Context {
    pub request: Request,
    pub envs: BTreeMap<String, EnvironmentItemValue>,
    pub logger: Logger,
}

#[derive(Default, Clone)]
pub struct Logger {
    infos: Vec<String>,
    errors: Vec<String>,
}

impl Logger {
    pub fn add_log(&mut self, msg: String) {
        self.infos.push(msg)
    }
    pub fn add_error(&mut self, msg: String) {
        self.errors.push(msg)
    }

    pub fn infos(&self) -> Vec<String> {
        self.infos.clone()
    }
    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }
}

impl ScriptRuntime {
    pub fn run(&self, js: String, context: Context) -> Promise<Result<Context, Error>> {
        Promise::spawn_thread("script", || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async { ScriptRuntime::run_js(js, context).await })
        })
    }

    async fn run_js(js: String, context: Context) -> Result<Context, Error> {
        let runjs_extension = ExtensionBuilder::default()
            .ops(vec![
                set_env::DECL,
                get_env::DECL,
                add_params::DECL,
                add_header::DECL,
                log::DECL,
                error::DECL,
            ])
            .build();
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            extensions: vec![runjs_extension],
            ..Default::default()
        });
        js_runtime.op_state().borrow_mut().put(context);
        let runtime_init_code = include_str!("./resource/runtime.js");
        js_runtime
            .execute_script_static("[runjs:runtime.js]", runtime_init_code)
            .unwrap();
        let temp = Url::from_file_path(Path::new("/temp/script.js")).unwrap();
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
fn set_env(state: &mut OpState, #[string] key: String, #[string] value: String) {
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
            c.logger
                .infos
                .push(format!("set env: `{}` as `{}`", key, value));
        }
    }
}

#[op2]
#[string]
fn get_env(state: &mut OpState, #[string] key: String) -> String {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => "".to_string(),
        Some(c) => c
            .envs
            .get(key.as_str())
            .cloned()
            .unwrap_or_default()
            .value
            .clone(),
    }
}

#[op2(fast)]
fn add_header(state: &mut OpState, #[string] key: String, #[string] value: String) {
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
            c.logger
                .infos
                .push(format!("add header: `{}` as `{}`", key, value));
        }
    }
}

#[op2(fast)]
fn add_params(state: &mut OpState, #[string] key: String, #[string] value: String) {
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
            c.logger
                .infos
                .push(format!("add params: `{}` as `{}`", key, value));
        }
    }
}

#[op2(fast)]
fn log(state: &mut OpState, #[string] msg: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.logger.add_log(msg),
    }
}

#[op2(fast)]
fn error(state: &mut OpState, #[string] msg: String) {
    let context = state.try_borrow_mut::<Context>();
    match context {
        None => {}
        Some(c) => c.logger.add_error(msg),
    }
}
