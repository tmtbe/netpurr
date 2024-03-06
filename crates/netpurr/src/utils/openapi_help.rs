use openapiv3::{
    Header, ObjectType, OpenAPI, Operation, Parameter, ParameterData, ParameterSchemaOrContent,
    ReferenceOr, RequestBody, Response, Schema, SchemaKind, Type,
};
use serde_json::{json, Value};
use strum_macros::Display;

pub struct OpenApiHelp {
    pub openapi: OpenAPI,
}
pub trait GetItem<R> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<R>;
}
impl GetItem<Schema> for ReferenceOr<Schema> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<Schema> {
        return match self {
            ReferenceOr::Reference { reference } => {
                OpenApiHelp::get_schema_with_ref(openapi, reference.clone())
            }
            ReferenceOr::Item(s) => Some(s.clone()),
        };
    }
}
impl GetItem<Schema> for ReferenceOr<Box<Schema>> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<Schema> {
        return match self {
            ReferenceOr::Reference { reference } => {
                OpenApiHelp::get_schema_with_ref(openapi, reference.clone())
            }
            ReferenceOr::Item(s) => Some(*s.clone()),
        };
    }
}
impl GetItem<Parameter> for ReferenceOr<Parameter> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<Parameter> {
        return match self {
            ReferenceOr::Reference { reference } => {
                OpenApiHelp::get_parameter_with_ref(openapi, reference.clone())
            }
            ReferenceOr::Item(s) => Some(s.clone()),
        };
    }
}
impl GetItem<Parameter> for ReferenceOr<Box<Parameter>> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<Parameter> {
        return match self {
            ReferenceOr::Reference { reference } => {
                OpenApiHelp::get_parameter_with_ref(openapi, reference.clone())
            }
            ReferenceOr::Item(s) => Some(*s.clone()),
        };
    }
}
impl GetItem<RequestBody> for ReferenceOr<RequestBody> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<RequestBody> {
        return match self {
            ReferenceOr::Reference { reference } => {
                OpenApiHelp::get_request_body_with_ref(openapi, reference.clone())
            }
            ReferenceOr::Item(s) => Some(s.clone()),
        };
    }
}
impl GetItem<RequestBody> for ReferenceOr<Box<RequestBody>> {
    fn get_item(&self, openapi: &OpenAPI) -> Option<RequestBody> {
        return match self {
            ReferenceOr::Reference { reference } => {
                OpenApiHelp::get_request_body_with_ref(openapi, reference.clone())
            }
            ReferenceOr::Item(s) => Some(*s.clone()),
        };
    }
}
impl OpenApiHelp {
    fn get_schema_with_ref(openapi: &OpenAPI, ref_name: String) -> Option<Schema> {
        return match openapi.components.clone() {
            None => None,
            Some(c) => {
                let s_name = ref_name.trim_start_matches("#/components/schemas/");
                let find = c.schemas.get(s_name);
                match find {
                    None => None,
                    Some(rs) => rs.clone().into_item(),
                }
            }
        };
    }
    fn get_response_with_ref(openapi: &OpenAPI, ref_name: String) -> Option<Response> {
        return match openapi.components.clone() {
            None => None,
            Some(c) => {
                let s_name = ref_name.trim_start_matches("#/components/responses/");
                let find = c.responses.get(s_name);
                match find {
                    None => None,
                    Some(rs) => rs.clone().into_item(),
                }
            }
        };
    }
    fn get_parameter_with_ref(openapi: &OpenAPI, ref_name: String) -> Option<Parameter> {
        return match openapi.components.clone() {
            None => None,
            Some(c) => {
                let s_name = ref_name.trim_start_matches("#/components/Parameters/");
                let find = c.parameters.get(s_name);
                match find {
                    None => None,
                    Some(rs) => rs.clone().into_item(),
                }
            }
        };
    }
    fn get_request_body_with_ref(openapi: &OpenAPI, ref_name: String) -> Option<RequestBody> {
        return match openapi.components.clone() {
            None => None,
            Some(c) => {
                let s_name = ref_name.trim_start_matches("#/components/request_bodies/");
                let find = c.request_bodies.get(s_name);
                match find {
                    None => None,
                    Some(rs) => rs.clone().into_item(),
                }
            }
        };
    }
    fn get_header_with_ref(openapi: &OpenAPI, ref_name: String) -> Option<Header> {
        return match openapi.components.clone() {
            None => None,
            Some(c) => {
                let s_name = ref_name.trim_start_matches("#/components/headers/");
                let find = c.headers.get(s_name);
                match find {
                    None => None,
                    Some(rs) => rs.clone().into_item(),
                }
            }
        };
    }
}
impl OpenApiHelp {
    pub fn get_operation(&self, operation_id: String) -> Option<Operation> {
        for (_, path_item) in self.openapi.paths.iter() {
            if let Some(item) = path_item.as_item() {
                let mut ops: Vec<Option<Operation>> = vec![];
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
                                return Some(options.clone());
                            }
                        }
                    }
                }
            }
        }
        return None;
    }
    pub fn gen_openapi_schema(&self, operation_id: String) -> Option<Value> {
        if let Some(options) = self.get_operation(operation_id.clone()) {
            if let Some(op_id) = options.operation_id.clone() {
                if op_id == operation_id {
                    if let Some(request_body) = options.request_body.clone() {
                        if let Some(rb) = request_body.get_item(&self.openapi) {
                            if let Some(mt) = rb.content.get("application/json") {
                                if let Some(rs) = &mt.schema {
                                    if let Some(s) = rs.get_item(&self.openapi) {
                                        return self.gen_schema(s);
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

    pub fn gen_schema(&self, s: Schema) -> Option<Value> {
        return match s.schema_kind.clone() {
            SchemaKind::Type(t) => match t {
                Type::Object(ot) => {
                    let mut json_tree = json!({});
                    for (name, rs) in ot.properties.iter() {
                        if let Some(s) = rs.get_item(&self.openapi) {
                            let json_child = self.gen_schema(s);
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
                    }
                    Some(json_tree)
                }
                Type::Array(at) => {
                    let mut json_tree = json!([]);
                    match at.items {
                        None => {}
                        Some(rs) => {
                            if let Some(s) = rs.get_item(&self.openapi) {
                                let json_child = self.gen_schema(s);
                                match json_child {
                                    None => {}
                                    Some(child) => json_tree.as_array_mut().unwrap().push(child),
                                }
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
