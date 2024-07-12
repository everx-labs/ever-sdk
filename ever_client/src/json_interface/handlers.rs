/*
 * Copyright 2018-2021 EverX Labs Ltd.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific EVERX DEV software governing permissions and
 * limitations under the License.
 *
 */

use crate::client::{AppObject, ClientContext, Error};
use crate::error::ClientResult;
use crate::json_interface::runtime::Runtime;
use api_info::{ApiType, Field, Type};
use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::marker::PhantomData;
use std::sync::Arc;
use super::request::Request;
use super::runtime::{AsyncHandler, SyncHandler};

const ENUM_TYPE_TAG: &str = "type";
const ENUM_VALUE_FIELD: &str = "value";

fn parse_params<P: DeserializeOwned + ApiType>(params_json: &str) -> ClientResult<P> {
    match serde_json::from_str(params_json) {
        Ok(deserialized) => Ok(deserialized),
        Err(err) => {
            let mut error = Error::invalid_params(params_json, err);
            if let Ok(value) = serde_json::from_str::<Value>(params_json) {
                let field = P::api();
                let mut errors = vec![];
                let mut suggest_use_helper_for = vec![];
                check_params_for_known_errors(
                    &ProcessingPath::default().append(&field.name),
                    &field,
                    Some(&value),
                    &mut errors,
                    &mut suggest_use_helper_for,
                );
                for error_message in errors.iter() {
                    error.message.push_str(&format!("\nTip: {}", error_message));
                }
                if suggest_use_helper_for.len() > 0 {
                    error.data["suggest_use_helper_for"] = Value::Array(
                        suggest_use_helper_for.iter()
                            .map(|s| Value::String(s.to_string()))
                            .collect()
                    );
                }
            } else {
                error.message.push_str("\nTip: Fix syntax error in the JSON string.");
            }

            Err(error)
        }
    }
}

#[derive(Default, Clone)]
struct ProcessingPath {
    path: Vec<String>,
}

impl ProcessingPath {
    fn append(&self, field_name: &str) -> Self {
        let mut path = self.path.clone();
        if field_name.len() > 0 {
            path.push(field_name.to_string());
        } else {
            path.push(ENUM_VALUE_FIELD.to_string());
        }

        Self { path }
    }

    fn resolve_field_name(&self) -> &str {
        self.path.last()
            .map(|string| string.as_str())
            .unwrap_or("<unresolved>")
    }
}

fn check_params_for_known_errors(
    path: &ProcessingPath,
    mut field: &Field,
    value: Option<&Value>,
    errors: &mut Vec<String>,
    suggest_use_helper_for: &mut Vec<&'static str>,
) {
    let mut class_name = None;
    while let Type::Ref { ref name } = field.value {
        if let Some(field_ref) = Runtime::api().find_type(&name) {
            field = field_ref;
            class_name = Some(name.as_str());
        }
    };

    let value = match &field.value {
        Type::Optional { inner } => {
            if let Some(value) = value {
                check_type(path, &class_name, &inner, value, errors, suggest_use_helper_for);
            }
            return;
        }
        _ => {
            match value {
                Some(value) => value,
                None => {
                    errors.push(
                        format!(
                            r#"Field "{}" value is expected, but not provided."#,
                            path.resolve_field_name(),
                        )
                    );
                    return;
                },
            }
        }
    };

    check_type(path, &class_name, &field.value, value, errors, suggest_use_helper_for);
}

fn check_type(
    path: &ProcessingPath,
    class_name: &Option<&str>,
    field_type: &Type,
    value: &Value,
    errors: &mut Vec<String>,
    suggest_use_helper_for: &mut Vec<&'static str>,
) {
    match field_type {
        Type::Array { item } => {
            if let Value::Array(ref vec) = value {
                for index in 0..vec.len() {
                    check_type(
                        &path.append(&format!("{}[{}]", path.resolve_field_name(), index)),
                        class_name,
                        &item,
                        &vec[index],
                        errors,
                        suggest_use_helper_for,
                    );
                }
            } else {
                errors.push(
                    format!(
                        "Field \"{}\" is expected to be an array, but actual value is {:?}.",
                        path.resolve_field_name(),
                        value,
                    )
                );
            }
        }
        Type::Struct { ref fields } => {
            if let Value::Object(map) = value {
                for struct_field in fields {
                    check_params_for_known_errors(
                        &path.append(&struct_field.name),
                        struct_field,
                        map.get(&struct_field.name),
                        errors,
                        suggest_use_helper_for,
                    )
                }
            } else {
                errors.push(
                    format!(
                        "Field \"{}\" is expected to be an object, but actual value is {:?}.",
                        path.resolve_field_name(),
                        value,
                    )
                );
            }
        }
        Type::EnumOfTypes { types } => {
            if let Value::Object(map) = value {
                if let Some(type_name) = map.get(ENUM_TYPE_TAG) {
                    let type_name = match type_name.as_str() {
                        Some(type_name) => type_name,
                        None => {
                            errors.push(
                                format!("Field \"{}\" is expected to be `String`.", ENUM_TYPE_TAG)
                            );
                            return;
                        }
                    };
                    if let Some(enum_type) = types.iter().find(|item| item.name == type_name) {
                        check_params_for_known_errors(
                            &path,
                            enum_type,
                            Some(value),
                            errors,
                            suggest_use_helper_for,
                        );
                        return;
                    }
                }
            }
            get_incorrect_enum_errors(
                path.resolve_field_name(),
                class_name,
                types,
                errors,
                suggest_use_helper_for,
            )
        }

        _ => {
            // Skip unsupported
        }
    }
}

fn get_incorrect_enum_errors(
    field_name: &str,
    class_name: &Option<&str>,
    types: &Vec<Field>,
    errors: &mut Vec<String>,
    suggest_use_helper_for: &mut Vec<&'static str>,
) {
    let types_str = types.iter()
        .map(|field| format!(r#""{}""#, field.name))
        .collect::<Vec<String>>()
        .join(", ");

    static SUGGEST_USE_HELPER_FOR_SORTED: &[&'static str] = &["Abi", "Signer"];

    errors.push(format!(
        "Field \"{field}\" must be a structure:\n\
        {{\n    \
            \"{type_tag}\": one of {types},\n    \
            ... fields of a corresponding structure, or \"value\" in a case of scalar\n\
        }}.",
        field = field_name,
        type_tag = ENUM_TYPE_TAG,
        types = types_str,
    ));

    let class_name = match *class_name {
        Some(class_name) => class_name,
        None => field_name,
    };
    if let Ok(index) = SUGGEST_USE_HELPER_FOR_SORTED.binary_search(&class_name) {
        suggest_use_helper_for.push(SUGGEST_USE_HELPER_FOR_SORTED[index])
    }
}

pub(crate) struct SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut)>>,
}

impl<P, R, Fut, F> SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<P, R, Fut, F> AsyncHandler for SpawnHandlerCallback<P, R, Fut, F>
where
    P: Send + DeserializeOwned + ApiType + 'static,
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P, Arc<Request>) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();

        context.env.spawn(async move {
            let request = Arc::new(request);
            match parse_params(&params_json) {
                Ok(params) => {
                    let result = handler(context_copy, params, request.clone()).await;
                    request.response_result(result);
                }
                Err(err) => request.finish_with_error(err),
            };
        });
    }
}

pub(crate) struct SpawnHandlerAppObject<P, R, Fut, F, AP, AR>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    AP: Send + Serialize + 'static,
    AR: Send + DeserializeOwned + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, AppObject<AP, AR>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut, AP, AR)>>,
}

impl<P, R, Fut, F, AP, AR> SpawnHandlerAppObject<P, R, Fut, F, AP, AR>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    AP: Send + Serialize + 'static,
    AR: Send + DeserializeOwned + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P, AppObject<AP, AR>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<P, R, Fut, F, AP, AR> AsyncHandler for SpawnHandlerAppObject<P, R, Fut, F, AP, AR>
where
    P: Send + DeserializeOwned + ApiType + 'static,
    R: Send + Serialize + 'static,
    AP: Send + Serialize + 'static,
    AR: Send + DeserializeOwned + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P, AppObject<AP, AR>) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            let request = Arc::new(request);
            match parse_params(&params_json) {
                Ok(params) => {
                    let app_object = AppObject::new(context_copy.clone(), request.clone());
                    let result = handler(context_copy, params, app_object).await;
                    request.response_result(result);
                }
                Err(err) => request.finish_with_error(err),
            };
        });
    }
}

pub(crate) struct SpawnHandlerAppObjectNoArgs<R, Fut, F, AP, AR>
where
    R: Send + Serialize + 'static,
    AP: Send + Serialize + 'static,
    AR: Send + DeserializeOwned + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, AppObject<AP, AR>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(R, Fut, AP, AR)>>,
}

impl<R, Fut, F, AP, AR> SpawnHandlerAppObjectNoArgs<R, Fut, F, AP, AR>
where
    R: Send + Serialize + 'static,
    AP: Send + Serialize + 'static,
    AR: Send + DeserializeOwned + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, AppObject<AP, AR>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<R, Fut, F, AP, AR> AsyncHandler for SpawnHandlerAppObjectNoArgs<R, Fut, F, AP, AR>
where
    R: Send + Serialize + 'static,
    AP: Send + Serialize + 'static,
    AR: Send + DeserializeOwned + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, AppObject<AP, AR>) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, _params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            let request = Arc::new(request);
            let app_object = AppObject::new(context_copy.clone(), request.clone());
            let result = handler(context_copy, app_object).await;
            request.response_result(result);
        });
    }
}

pub(crate) struct SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(P, R, Fut)>>,
}

impl<P, R, Fut, F> SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + 'static,
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<P, R, Fut, F> AsyncHandler for SpawnHandler<P, R, Fut, F>
where
    P: Send + DeserializeOwned + ApiType + 'static,
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>, P) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            match parse_params(&params_json) {
                Ok(params) => {
                    let result = handler(context_copy, params).await;
                    request.finish_with_result(result);
                }
                Err(err) => request.finish_with_error(err),
            };
        });
    }
}

pub(crate) struct SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    handler: Arc<F>,
    // Mutex is needed to have Sync trait implemented for struct
    phantom: PhantomData<std::sync::Mutex<(R, Fut)>>,
}

impl<R, Fut, F> SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Future<Output = ClientResult<R>> + 'static,
    F: Send + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler: Arc::new(handler),
            phantom: PhantomData,
        }
    }
}

impl<R, Fut, F> AsyncHandler for SpawnNoArgsHandler<R, Fut, F>
where
    R: Send + Serialize + 'static,
    Fut: Send + Future<Output = ClientResult<R>> + 'static,
    F: Send + Sync + Fn(Arc<ClientContext>) -> Fut + 'static,
{
    fn handle(&self, context: Arc<ClientContext>, _params_json: String, request: Request) {
        let handler = self.handler.clone();
        let context_copy = context.clone();
        context.env.spawn(async move {
            request.finish_with_result(handler(context_copy).await);
        });
    }
}

pub(crate) struct CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ClientResult<R>,
{
    handler: F,
    phantom: PhantomData<std::sync::Mutex<(P, R)>>,
}

impl<P, R, F> CallHandler<P, R, F>
where
    P: Send + DeserializeOwned,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ClientResult<R>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<P, R, F> SyncHandler for CallHandler<P, R, F>
where
    P: Send + DeserializeOwned + ApiType,
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>, P) -> ClientResult<R>,
{
    fn handle(&self, context: Arc<ClientContext>, params_json: &str) -> ClientResult<String> {
        match parse_params(params_json) {
            Ok(params) => (self.handler)(context, params).and_then(|x| {
                serde_json::to_string(&x).map_err(|err| Error::cannot_serialize_result(err))
            }),
            Err(err) => Err(err),
        }
    }
}

pub(crate) struct CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ClientResult<R>,
{
    handler: F,
    phantom: PhantomData<std::sync::Mutex<R>>,
}

impl<R, F> CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ClientResult<R>,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<R, F> SyncHandler for CallNoArgsHandler<R, F>
where
    R: Send + Serialize,
    F: Fn(Arc<ClientContext>) -> ClientResult<R>,
{
    fn handle(&self, context: Arc<ClientContext>, _params_json: &str) -> ClientResult<String> {
        match (self.handler)(context) {
            Ok(result) => {
                serde_json::to_string(&result).map_err(|err| Error::cannot_serialize_result(err))
            }
            Err(err) => Err(err),
        }
    }
}
