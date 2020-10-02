use crate::client::client::get_handlers;
use api_doc::api::{Field, Method, Type, API};
use std::collections::HashMap;

pub trait CoreModuleInfo {
    fn name() -> &str;
}

pub fn get_api() -> API {
    ApiBuilder::new().build()
}

fn split_name(name: &str) -> (Option<String>, String) {
    let mut parts = name.split(".");
    let a = parts.next();
    let b = parts.next();
    if let Some(b) = b {
        (a.map(|x| x.to_string()), b.to_string())
    } else {
        (None, name.to_string())
    }
}

fn is_full_name(name: &str) -> bool {
    name.contains(".")
}

fn full_name(module: &Option<String>, name: &str) -> String {
    if let Some(module) = module {
        format!("{}.{}", module, name)
    } else {
        name.to_string()
    }
}

pub(crate) struct ApiBuilder<'a> {
    api: &'a API,
    names: HashMap<String, Vec<String>>,
}

impl<'a> ApiBuilder<'a> {
    pub(crate) fn new() -> Self {
        let api = &get_handlers().api;
        let mut builder = Self {
            api,
            names: HashMap::new(),
        };
        builder.init();
        builder
    }

    fn init(&mut self) {
        for ty in &self.api.types {
            let (module, name) = split_name(&ty.name);
            self.reg_type(&name, ty);
            self.reg_type(&full_name(&module, &name), ty);
        }
    }

    fn reg_type(&mut self, name: &str, ty: &Field) {
        if let Some(existing) = self.names.get_mut(name) {
            existing.push(ty.name.clone());
        } else {
            self.names.insert(name.to_string(), vec![ty.name.clone()]);
        }
    }


    pub(crate) fn build(&self) -> API {
        let mut reduced = API::default();
        reduced.types = self.api.types.clone();
        reduced.methods = self
            .api
            .methods
            .iter()
            .map(|x| self.reduce_method(x))
            .collect();
        reduced
    }

    fn resolve_refs(&self, def_module: &Option<String>, ty: &mut Type) {
        match ty {
            Type::Ref(name) => {
                if !is_full_name(name) {
                    let full = full_name(def_module, name);
                    if self.names.contains_key(&full) {
                        *name = full;
                    } else if let Some(names) = self.names.get(name) {
                        *name = names
                            .iter()
                            .find(|x| is_full_name(x))
                            .map(|x| x)
                            .unwrap_or(&names[0])
                            .clone();
                    }
                }
            }
            Type::Generic { args, .. } => {
                for ty in args {
                    self.resolve_refs(def_module, ty);
                }
            }
            Type::Array(item) => self.resolve_refs(def_module, item),
            Type::EnumOfTypes(vars) => {
                for ty in vars {
                    self.resolve_refs(def_module, &mut ty.value);
                }
            }
            Type::Optional(ty) => self.resolve_refs(def_module, ty),
            Type::Struct(fields) => {
                for ty in fields {
                    self.resolve_refs(def_module, &mut ty.value);
                }
            }
            _ => {}
        }
    }

    fn reduce_method(&self, method: &Method) -> Method {
        let (module, _) = split_name(&method.name);
        let mut params = method
            .params
            .iter()
            .find(|x| x.name == "params")
            .map(|x| vec![x.clone()])
            .unwrap_or(vec![]);
        if params.len() > 0 {
            self.resolve_refs(&module, &mut params[0].value);
        }
        let mut result = match &method.result {
            Type::Generic { name, args } if name == "ApiResult" => args[0].clone(),
            _ => method.result.clone(),
        };
        self.resolve_refs(&module, &mut result);
        let mut reduced = Method {
            name: method.name.clone(),
            params,
            result,
            summary: method.summary.clone(),
            description: method.description.clone(),
            errors: method.errors.clone(),
        };
        if reduced.params.len() > 0 && reduced.params[0].name == "context" {
            reduced.params.remove(0);
        }
        reduced
    }
}
