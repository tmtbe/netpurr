use std::pin::Pin;

use data_url::DataUrl;
use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::anyhow::{bail, Error};
use deno_core::futures::FutureExt;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::{resolve_import, ModuleSourceCode};
use deno_core::{ModuleCode, ModuleSource};
use deno_core::{ModuleLoader, ResolutionKind};
use log::info;
use url::Url;

pub struct SimpleModuleLoader;

impl ModuleLoader for SimpleModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        let string_specifier = module_specifier.to_string();
        async move {
            info!("start load module: {}", string_specifier);
            let mut module_url_found = string_specifier.clone();
            let bytes = match module_specifier.scheme() {
                "http" | "https" => {
                    let res = reqwest::get(module_specifier).await?;
                    let res = res.error_for_status()?;
                    // res.url() is the post-redirect URL.
                    module_url_found = res.url().to_string();
                    res.bytes().await?.to_vec()
                }
                "file" => {
                    let path = match module_specifier.to_file_path() {
                        Ok(path) => path,
                        Err(_) => bail!("Invalid file URL."),
                    };
                    tokio::fs::read(path).await?
                }
                "data" => {
                    let url = match DataUrl::process(module_specifier.as_str()) {
                        Ok(url) => url,
                        Err(_) => bail!("Not a valid data URL."),
                    };
                    match url.decode_to_vec() {
                        Ok((bytes, _)) => bytes,
                        Err(_) => bail!("Not a valid data URL."),
                    }
                }
                schema => bail!("Invalid schema {}", schema),
            };
            info!("load module: {} success", string_specifier);
            let media_type = MediaType::from_str(string_specifier.as_str());
            let (module_type, should_transpile) = match &media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (deno_core::ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (ModuleType::JavaScript, true),
                MediaType::Json => (ModuleType::Json, false),
                _ => (ModuleType::JavaScript, true),
            };
            let code = String::from_utf8(bytes)?;
            let code = if should_transpile {
                info!("start transpile module: {}", string_specifier);
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: string_specifier.clone(),
                    text_info: SourceTextInfo::from_string(code),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                let t_code = parsed.transpile(&Default::default())?.text;
                info!("transpile module success: {}", string_specifier);
                t_code
            } else {
                code
            };
            let specified = Url::parse(string_specifier.as_str()).unwrap();
            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(ModuleCode::from(code)),
                &specified,
            ))
        }
        .boxed_local()
    }
}
