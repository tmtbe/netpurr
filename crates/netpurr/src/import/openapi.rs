use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::string::ToString;

use anyhow::anyhow;
use openapiv3::{
    MediaType, OpenAPI, Operation, Parameter, ReferenceOr, RequestBody, Schema, SchemaKind,
    StringFormat, Tag, Type, VariantOrUnknownOrEmpty,
};
use regex::Regex;

use netpurr_core::data::auth::Auth;
use netpurr_core::data::collections::{Collection, CollectionFolder};
use netpurr_core::data::environment::{EnvironmentConfig, EnvironmentItem};
use netpurr_core::data::http::{
    BodyRawType, BodyType, Header, HttpBody, HttpRecord, Method, MultipartData, MultipartDataType,
    QueryParam, Request, RequestSchema,
};
use netpurr_core::data::record::Record;

const DEFAULT_TAG: &str = "_Default";

pub struct OpenApi {
    source: OpenAPI,
}

pub struct OpenApiOperation {
    operation: Operation,
    method: Method,
    path: String,
}

impl OpenApi {
    pub fn try_import(json: String) -> serde_json::Result<OpenApi> {
        let openapi: OpenAPI = serde_json::from_str(json.as_str())?;
        return Ok(OpenApi { source: openapi });
    }
    pub fn to_collection(&self) -> anyhow::Result<Collection> {
        if self.source.info.title.is_empty() {
            return Err(anyhow!("not postman"));
        }
        let mut tag_map: HashMap<String, Vec<OpenApiOperation>> = HashMap::new();
        tag_map.insert(DEFAULT_TAG.to_string(), vec![]);
        for (path, ref_path_item) in self.source.paths.iter() {
            if let Some(path_item) = ref_path_item.as_item() {
                if let Some(get) = path_item.get.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::GET, get);
                }
                if let Some(put) = path_item.put.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::PUT, put);
                }
                if let Some(delete) = path_item.delete.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::DELETE, delete);
                }
                if let Some(post) = path_item.post.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::POST, post);
                }
                if let Some(patch) = path_item.patch.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::PATCH, patch);
                }
                if let Some(options) = path_item.options.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::OPTIONS, options);
                }
                if let Some(head) = path_item.head.clone() {
                    Self::group_operation(&mut tag_map, path.clone(), Method::HEAD, head);
                }
            }
        }
        let collection = Collection {
            envs: EnvironmentConfig {
                items: vec![EnvironmentItem {
                    enable: true,
                    key: "server_host".to_string(),
                    value: "localhost:8080".to_string(),
                    desc: "".to_string(),
                    value_type: Default::default(),
                }],
            },
            openapi: Some(self.source.clone()),
            folder: Rc::new(RefCell::new(CollectionFolder {
                name: self.source.info.title.to_string(),
                parent_path: ".".to_string(),
                desc: self.source.info.description.clone().unwrap_or_default(),
                auth: Default::default(),
                is_root: true,
                requests: Self::gen_requests(
                    tag_map.get(DEFAULT_TAG).unwrap(),
                    RequestSchema::HTTP,
                ),
                folders: Self::gen_folders(tag_map, self.source.tags.clone()),
                pre_request_script: "".to_string(),
                test_script: "".to_string(),
            })),
        };

        Ok(collection)
    }

    fn group_operation(
        tag_map: &mut HashMap<String, Vec<OpenApiOperation>>,
        path: String,
        method: Method,
        op: Operation,
    ) {
        if op.tags.is_empty() {
            let mut operations = tag_map.get_mut(DEFAULT_TAG).unwrap();
            operations.push(OpenApiOperation {
                operation: op.clone(),
                method,
                path,
            })
        } else {
            let tag_name = op.tags.get(0).unwrap();
            if !tag_map.contains_key(tag_name) {
                tag_map.insert(tag_name.to_string(), vec![]);
            }
            let mut operations = tag_map.get_mut(tag_name).unwrap();
            operations.push(OpenApiOperation {
                operation: op.clone(),
                method,
                path,
            })
        }
    }
    pub fn gen_folders(
        tag_map: HashMap<String, Vec<OpenApiOperation>>,
        tags: Vec<Tag>,
    ) -> BTreeMap<String, Rc<RefCell<CollectionFolder>>> {
        let mut openapi_tags: HashMap<String, Tag> = HashMap::new();
        for tag in tags.iter() {
            openapi_tags.insert(tag.name.clone(), tag.clone());
        }
        let folders: Vec<CollectionFolder> = tag_map
            .iter()
            .filter(|(name, _)| {
                return name.as_str() != DEFAULT_TAG;
            })
            .map(|(name, records)| CollectionFolder {
                name: name.clone(),
                parent_path: "".to_string(),
                desc: openapi_tags
                    .get(name)
                    .cloned()
                    .unwrap()
                    .description
                    .unwrap_or_default(),
                auth: Default::default(),
                is_root: false,
                requests: Self::gen_requests(records, RequestSchema::HTTP),
                folders: Default::default(),
                pre_request_script: "".to_string(),
                test_script: "".to_string(),
            })
            .collect();
        let mut result = BTreeMap::default();
        for folder in folders.iter() {
            result.insert(folder.name.clone(), Rc::new(RefCell::new(folder.clone())));
        }
        result
    }
    pub fn gen_requests(
        operations: &Vec<OpenApiOperation>,
        schema: RequestSchema,
    ) -> BTreeMap<String, Record> {
        let http_records: Vec<Record> = operations
            .iter()
            .map(|op| {
                Record::Rest(HttpRecord {
                    name: op.operation.summary.clone().unwrap_or_default(),
                    desc: op.operation.description.clone().unwrap_or_default(),
                    operation_id: op.operation.operation_id.clone(),
                    request: Request {
                        method: op.method.clone(),
                        schema: schema.clone(),
                        raw_url: Self::gen_raw_path(op),
                        base_url: "".to_string(),
                        path_variables: vec![],
                        params: op
                            .operation
                            .parameters
                            .iter()
                            .map(|q| q.as_item())
                            .filter(|q| q.is_some())
                            .map(|q| q.unwrap())
                            .filter(|q| match q {
                                Parameter::Query { .. } => true,
                                _ => false,
                            })
                            .map(|q| QueryParam {
                                key: q.clone().parameter_data().name.clone(),
                                value: "".to_string(),
                                desc: q.clone().parameter_data().description.unwrap_or_default(),
                                lock_with: Default::default(),
                                enable: true,
                            })
                            .collect(),
                        headers: op
                            .operation
                            .parameters
                            .iter()
                            .map(|q| q.as_item())
                            .filter(|q| q.is_some())
                            .map(|q| q.unwrap())
                            .filter(|q| match q {
                                Parameter::Header { .. } => true,
                                _ => false,
                            })
                            .map(|q| Header {
                                key: q.clone().parameter_data().name.clone(),
                                value: "".to_string(),
                                desc: q.clone().parameter_data().description.unwrap_or_default(),
                                lock_with: Default::default(),
                                enable: true,
                            })
                            .collect(),
                        body: Self::gen_http_body(op.operation.request_body.clone()),
                        auth: Auth::default(),
                    },
                    ..Default::default()
                })
            })
            .collect();
        let mut result = BTreeMap::default();
        for http_record in http_records.iter() {
            let mut record_clone = http_record.clone();
            record_clone.must_get_mut_rest().request.parse_raw_url();
            result.insert(http_record.name(), record_clone);
        }
        result
    }

    fn gen_http_body(option: Option<ReferenceOr<RequestBody>>) -> HttpBody {
        let mut body = HttpBody::default();
        match option {
            None => body,
            Some(rr) => match rr.as_item() {
                None => body,
                Some(r) => {
                    for (name, mt) in r.content.iter() {
                        match name.to_lowercase().as_str() {
                            "application/json" => {
                                body.body_type = BodyType::RAW;
                                body.body_raw_type = BodyRawType::JSON
                            }
                            "multipart/form-data" => {
                                body.body_type = BodyType::FROM_DATA;
                                match mt.schema.clone() {
                                    None => {}
                                    Some(rs) => match rs.as_item() {
                                        None => {}
                                        Some(s) => match s.schema_kind.clone() {
                                            SchemaKind::Type(t) => match t {
                                                Type::Object(o) => {
                                                    for (name, rs) in o.properties.iter() {
                                                        match rs.as_item() {
                                                            None => {}
                                                            Some(s) => {
                                                                match s.schema_kind.clone() {
                                                                    SchemaKind::Type(t) => match t {
                                                                        Type::String(s) => {
                                                                            match s.format {
                                                                            VariantOrUnknownOrEmpty::Item(sf) => {
                                                                                match sf {
                                                                                    StringFormat::Binary => {
                                                                                        body.body_form_data.push(MultipartData {
                                                                                            data_type: MultipartDataType::FILE,
                                                                                            key: name.clone(),
                                                                                            value: "".to_string(),
                                                                                            desc: "".to_string(),
                                                                                            lock_with: Default::default(),
                                                                                            enable: false,
                                                                                        })
                                                                                    },
                                                                                    _=>{
                                                                                        body.body_form_data.push(MultipartData {
                                                                                            data_type: MultipartDataType::TEXT,
                                                                                            key: name.clone(),
                                                                                            value: "".to_string(),
                                                                                            desc: "".to_string(),
                                                                                            lock_with: Default::default(),
                                                                                            enable: false,
                                                                                        })
                                                                                    }
                                                                                }
                                                                            }
                                                                          _ => {}
                                                                        }
                                                                        }
                                                                        _ => {}
                                                                    },
                                                                    _ => {}
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            },
                                            _ => {}
                                        },
                                    },
                                }
                            }
                            _ => {}
                        }
                    }
                    return body;
                }
            },
        }
    }

    fn gen_raw_path(op: &OpenApiOperation) -> String {
        let head = "http://{{ server_host }}";
        let re = Regex::new(r"\{([^{}]+)}").unwrap();
        let replaced_path = re.replace_all(op.path.as_str(), ":$1");
        return format!("{}{}", head, replaced_path);
    }
}
