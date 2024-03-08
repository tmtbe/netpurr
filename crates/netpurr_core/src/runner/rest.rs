use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use anyhow::anyhow;
use reqwest::header::CONTENT_TYPE;
use reqwest::multipart::Part;
use reqwest::Method;
use reqwest::{multipart, Body, Client};
use tokio::fs::File;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::data::environment::EnvironmentItemValue;
use crate::data::http;
use crate::data::http::{
    BodyRawType, BodyType, Header, HttpBody, LockWith, MultipartDataType, PathVariables, QueryParam,
};
use crate::data::logger::Logger;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}

impl RestSender {
    pub async fn reqwest_async_send(
        request: http::Request,
        client: Client,
    ) -> anyhow::Result<(http::Request, http::Response)> {
        let reqwest_request = Self::build_reqwest_request(request.clone()).await?;
        let mut new_request = request.clone();
        for (hn, hv) in reqwest_request.headers().iter() {
            if new_request
                .headers
                .iter()
                .find(|h| {
                    h.key.to_lowercase() == hn.to_string().to_lowercase()
                        && h.value == hv.to_str().unwrap_or("")
                })
                .is_none()
            {
                new_request.headers.push(Header {
                    key: hn.to_string(),
                    value: hv.to_str().unwrap_or("").to_string(),
                    desc: "auto gen".to_string(),
                    enable: true,
                    lock_with: LockWith::LockWithAuto,
                })
            }
        }
        let start_time = Instant::now();
        let reqwest_response = client.execute(reqwest_request).await?;
        let total_time = start_time.elapsed();
        Ok((
            new_request,
            http::Response {
                request: request.clone(),
                headers: Header::new_from_map(reqwest_response.headers()),
                status: reqwest_response.status().as_u16(),
                status_text: reqwest_response.status().to_string(),
                elapsed_time: total_time.as_millis(),
                logger: Logger::default(),
                body: Arc::new(HttpBody::new(reqwest_response.bytes().await?.to_vec())),
            },
        ))
    }

    pub async fn build_reqwest_request(request: http::Request) -> anyhow::Result<reqwest::Request> {
        let client = Client::new();
        let method = Method::from_str(request.method.to_string().to_uppercase().as_str())
            .unwrap_or_default();
        let _ = Uri::try_from(request.get_url_with_schema())?;
        let mut builder = client.request(method, request.get_url_with_schema());
        for header in request.headers.iter().filter(|h| h.enable) {
            builder = builder.header(header.key.clone(), header.value.clone());
        }
        let query: Vec<(String, String)> = request
            .params
            .iter()
            .filter(|q| q.enable)
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect();
        builder = builder.query(&query);
        match request.body.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                let mut form = multipart::Form::new();
                for md in request.body.body_form_data.iter().filter(|md| md.enable) {
                    match md.data_type {
                        MultipartDataType::FILE => {
                            let path = Path::new(md.value.as_str());
                            let content_type = mime_guess::from_path(path);
                            // read file body stream
                            let file = File::open(path).await?;
                            let stream = FramedRead::new(file, BytesCodec::new());
                            let file_body = Body::wrap_stream(stream);
                            //make form part of file
                            let fname = path
                                .file_name()
                                .unwrap()
                                .to_os_string()
                                .into_string()
                                .ok()
                                .unwrap();
                            let some_file = Part::stream(file_body).file_name(fname).mime_str(
                                content_type.first_or_octet_stream().to_string().as_str(),
                            )?;
                            form = form.part(md.key.clone(), some_file);
                        }
                        MultipartDataType::TEXT => {
                            form = form.text(md.key.clone(), md.value.clone());
                        }
                    }
                }
                builder = builder.multipart(form);
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                let mut params = HashMap::new();
                for md in request.body.body_xxx_form.iter().filter(|md| md.enable) {
                    params.insert(md.key.clone(), md.value.clone());
                }
                builder = builder.form(&params);
            }
            BodyType::RAW => match request.body.body_raw_type {
                BodyRawType::TEXT => {
                    builder = builder.header(CONTENT_TYPE, "text/plain");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::JSON => {
                    builder = builder.header(CONTENT_TYPE, "application/json");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::HTML => {
                    builder = builder.header(CONTENT_TYPE, "text/html");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::XML => {
                    builder = builder.header(CONTENT_TYPE, "application/xml");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::JavaScript => {
                    builder = builder.header(CONTENT_TYPE, "application/javascript");
                    builder = builder.body(request.body.body_str);
                }
            },
            BodyType::BINARY => {
                let path = Path::new(request.body.body_file.as_str());
                let content_type = mime_guess::from_path(path);
                builder = builder.header(
                    CONTENT_TYPE,
                    content_type.first_or_octet_stream().to_string(),
                );
                let file = File::open(path).await?;
                let stream = FramedRead::new(file, BytesCodec::new());
                let body = Body::wrap_stream(stream);
                builder = builder.body(body);
            }
        }
        match builder.build() {
            Ok(r) => Ok(r),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub(crate) fn build_request(
        request: http::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> http::Request {
        let mut build_request = request.clone();
        build_request.params = Self::build_query_params(request.params.clone(), &envs);
        build_request.base_url = Self::build_base_url(
            request.base_url.clone(),
            request.path_variables.clone(),
            &envs,
        );
        build_request.headers = Self::build_header(request.headers.clone(), &envs);
        build_request.body.body_str =
            crate::utils::replace_variable(build_request.body.body_str, envs.clone());
        for md in build_request.body.body_xxx_form.iter_mut() {
            md.key = crate::utils::replace_variable(md.key.clone(), envs.clone());
            md.value = crate::utils::replace_variable(md.value.clone(), envs.clone());
        }
        for md in build_request.body.body_form_data.iter_mut() {
            md.key = crate::utils::replace_variable(md.key.clone(), envs.clone());
            md.value = crate::utils::replace_variable(md.value.clone(), envs.clone());
        }
        build_request
    }

    fn build_header(
        headers: Vec<Header>,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> Vec<Header> {
        headers
            .iter()
            .filter(|h| h.enable)
            .map(|h| Header {
                key: h.key.clone(),
                value: crate::utils::replace_variable(h.value.clone(), envs.clone()),
                desc: h.desc.clone(),
                enable: h.enable,
                lock_with: h.lock_with.clone(),
            })
            .collect()
    }

    fn build_query_params(
        query_params: Vec<QueryParam>,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> Vec<QueryParam> {
        query_params
            .iter()
            .filter(|q| q.enable)
            .map(|q| QueryParam {
                key: q.key.clone(),
                value: crate::utils::replace_variable(q.value.clone(), envs.clone()),
                desc: q.desc.clone(),
                enable: q.enable,
                lock_with: q.lock_with.clone(),
            })
            .collect()
    }

    fn build_base_url(
        mut base_url: String,
        path_variables: Vec<PathVariables>,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> String {
        base_url = crate::utils::replace_variable(base_url, envs.clone());
        let build_path_variables: Vec<PathVariables> = path_variables
            .iter()
            .map(|p| PathVariables {
                key: p.key.clone(),
                value: crate::utils::replace_variable(p.value.clone(), envs.clone()),
                desc: p.desc.clone(),
            })
            .collect();
        let build_base_url: Vec<String> = base_url
            .split("/")
            .map(|part| {
                if part.starts_with(":") {
                    let variable = &part[1..];
                    build_path_variables
                        .iter()
                        .find(|v| v.key == variable)
                        .cloned()
                        .unwrap_or_default()
                        .value
                } else {
                    part.to_string()
                }
            })
            .collect();
        build_base_url.join("/")
    }
}
