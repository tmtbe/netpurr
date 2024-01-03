use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

use deno_core::anyhow::Error;
use deno_core::url::Url;
use deno_core::{op2, ExtensionBuilder, Op, OpState};
use deno_core::{ModuleCode, PollEventLoopOptions};

use crate::data::{EnvironmentItemValue, EnvironmentValueType, Header, QueryParam, Request};

pub struct ScriptRuntime {
    runtime: tokio::runtime::Runtime,
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        ScriptRuntime {
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub request: Request,
    pub envs: BTreeMap<String, EnvironmentItemValue>,
}

impl ScriptRuntime {
    pub fn run(&self, js: &'static str, context: Context) -> Result<Context, Error> {
        let new_context = self.runtime.block_on(self.run_js(js, context))?;
        Ok(new_context)
    }

    async fn run_js(&self, js: &'static str, context: Context) -> Result<Context, Error> {
        let runjs_extension = ExtensionBuilder::default()
            .ops(vec![
                set_env::DECL,
                get_env::DECL,
                add_params::DECL,
                add_header::DECL,
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
            .load_main_module(&temp, Some(ModuleCode::from_static(js)))
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
                key,
                EnvironmentItemValue {
                    value,
                    scope: "Script".to_string(),
                    value_type: EnvironmentValueType::String,
                },
            );
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
                key,
                value,
                enable: true,
                ..Default::default()
            });
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
                key,
                value,
                enable: true,
                ..Default::default()
            });
        }
    }
}
