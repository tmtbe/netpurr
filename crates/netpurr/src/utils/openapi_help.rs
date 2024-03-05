use egui::{Align, Button, Layout, Ui};
use egui_json_tree::{DefaultExpand, JsonTree};
use openapiv3::Type::Object;
use openapiv3::{ObjectType, OpenAPI, ReferenceOr, RequestBody, Schema, SchemaKind, Type};
use serde_json::{json, Value};

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};

pub struct OpenApiHelp {
    pub openapi: OpenAPI,
}

impl OpenApiHelp {
    pub fn gen_openapi_schema(self, operation_id: String) -> Option<Value> {
        for (_, path_item) in self.openapi.paths.iter() {
            if let Some(item) = path_item.as_item() {
                let mut ops: Vec<Option<openapiv3::Operation>> = vec![];
                ops.push(item.options.clone());
                ops.push(item.get.clone());
                ops.push(item.post.clone());
                ops.push(item.put.clone());
                ops.push(item.delete.clone());
                ops.push(item.patch.clone());
                ops.push(item.head.clone());
                ops.push(item.trace.clone());
                for op in ops.iter() {
                    if let Some(options) = op {
                        if let Some(op_id) = options.operation_id.clone() {
                            if op_id == operation_id {
                                if let Some(request_body) = options.request_body.clone() {
                                    match request_body {
                                        ReferenceOr::Reference { reference } => {}
                                        ReferenceOr::Item(rb) => {
                                            if let Some(mt) = rb.content.get("application/json") {
                                                if let Some(schema) = &mt.schema {
                                                    return self
                                                        .gen_schema(self.get_schema(schema));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return None;
    }

    pub(crate) fn get_schema(&self, rs: &ReferenceOr<Schema>) -> Schema {
        return match rs {
            ReferenceOr::Reference { reference } => self.get_schema_with_ref(reference.clone()),
            ReferenceOr::Item(s) => s.clone(),
        };
    }
    fn get_schema_box(&self, rs: &ReferenceOr<Box<Schema>>) -> Schema {
        return match rs {
            ReferenceOr::Reference { reference } => self.get_schema_with_ref(reference.clone()),
            ReferenceOr::Item(s) => *s.clone(),
        };
    }

    fn get_schema_with_ref(&self, ref_name: String) -> Schema {
        let default = Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(Object(ObjectType::default())),
        };
        return match self.openapi.components.clone() {
            None => default,
            Some(c) => {
                let s_name = ref_name.trim_start_matches("#/components/schemas/");
                let find = c.schemas.get(s_name);
                match find {
                    None => default,
                    Some(rs) => self.get_schema(rs),
                }
            }
        };
    }

    pub fn gen_schema(&self, s: Schema) -> Option<Value> {
        return match s.schema_kind.clone() {
            SchemaKind::Type(t) => match t {
                Type::Object(ot) => {
                    let mut json_tree = json!({});
                    for (name, rs) in ot.properties.iter() {
                        let json_child = self.gen_schema(self.get_schema_box(rs));
                        match json_child {
                            None => {}
                            Some(child) => {
                                json_tree
                                    .as_object_mut()
                                    .unwrap()
                                    .insert(name.clone(), child);
                            }
                        }
                    }
                    Some(json_tree)
                }
                Type::Array(at) => {
                    let mut json_tree = json!([]);
                    match at.items {
                        None => {}
                        Some(rs) => {
                            let json_child = self.gen_schema(self.get_schema_box(&rs));
                            match json_child {
                                None => {}
                                Some(child) => json_tree.as_array_mut().unwrap().push(child),
                            }
                        }
                    }
                    Some(json_tree)
                }
                Type::String(_) => Some(json!("string")),
                Type::Number(_) => Some(json!(10000)),
                Type::Integer(_) => Some(json!(10)),
                Type::Boolean(_) => Some(json!(true)),
            },
            _ => None,
        };
    }
}
