use std::path::Path;
use std::rc::Rc;

use deno_core::anyhow::Error;
use deno_core::url::Url;
use deno_core::{op2, ExtensionBuilder, Op};
use deno_core::{ModuleCode, PollEventLoopOptions};

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

impl ScriptRuntime {
    pub fn run(&self) -> Result<(), Error> {
        self.runtime
            .block_on(self.run_js("poster.set_env(\"1\",\"2\",\"3\")"))
    }

    async fn run_js(&self, js: &'static str) -> Result<(), Error> {
        let runjs_extension = ExtensionBuilder::default().ops(vec![set_env::DECL]).build();
        let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
            extensions: vec![runjs_extension],
            ..Default::default()
        });
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
        result.await
    }
}

#[op2(fast)]
fn set_env(#[string] name: String, #[string] key: String, #[string] value: String) {
    println!("{}-{}-{}", name, key, value)
}
