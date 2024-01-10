use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use reqwest::blocking::{multipart, Client};
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;

use crate::data::environment::EnvironmentItemValue;
use crate::data::http;
use crate::data::http::{BodyRawType, BodyType, Header, HttpBody, LockWith, MultipartDataType};
use crate::data::logger::Logger;
use crate::utils;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}

impl RestSender {
    pub fn reqwest_block_send(
        request: http::Request,
        client: Client,
    ) -> reqwest::Result<(http::Request, http::Response)> {
        let reqwest_request = Self::build_reqwest_request(request.clone())?;
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
        let reqwest_response = client.execute(reqwest_request)?;
        let total_time = start_time.elapsed();
        Ok((
            new_request,
            http::Response {
                headers: Header::new_from_map(reqwest_response.headers()),
                status: reqwest_response.status().as_u16(),
                status_text: reqwest_response.status().to_string(),
                elapsed_time: total_time.as_millis(),
                logger: Logger::default(),
                body: Arc::new(HttpBody::new(reqwest_response.bytes()?.to_vec())),
            },
        ))
    }

    pub fn build_reqwest_request(
        request: http::Request,
    ) -> reqwest::Result<reqwest::blocking::Request> {
        let client = Client::new();
        let method = Method::from_str(request.method.to_string().to_uppercase().as_str())
            .unwrap_or_default();
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
                        MultipartDataType::File => {
                            form = form
                                .file(md.key.clone(), Path::new(md.value.as_str()).to_path_buf())
                                .unwrap()
                        }
                        MultipartDataType::Text => {
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
                let file_name = path.file_name().and_then(|filename| filename.to_str());
                let mut file =
                    File::open(path).expect(format!("open {:?} error", file_name).as_str());
                let mut inner: Vec<u8> = vec![];
                io::copy(&mut file, &mut inner).expect("add_stream io copy error");
                builder = builder.body(inner);
            }
        }
        builder.build()
    }

    pub(crate) fn build_request(
        request: http::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> http::Request {
        let mut build_request = request.clone();
        build_request.headers = Self::build_header(request.headers.clone(), &envs);
        build_request.body.body_str =
            utils::replace_variable(build_request.body.body_str, envs.clone());
        for md in build_request.body.body_xxx_form.iter_mut() {
            md.key = utils::replace_variable(md.key.clone(), envs.clone());
            md.value = utils::replace_variable(md.value.clone(), envs.clone());
        }
        for md in build_request.body.body_form_data.iter_mut() {
            md.key = utils::replace_variable(md.key.clone(), envs.clone());
            md.value = utils::replace_variable(md.value.clone(), envs.clone());
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
                value: utils::replace_variable(h.value.clone(), envs.clone()),
                desc: h.desc.clone(),
                enable: h.enable,
                lock_with: h.lock_with.clone(),
            })
            .collect()
    }
}
