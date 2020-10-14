use api_info::{Function, Type, API};
use std::collections::HashMap;

fn is_full_name(name: &str) -> bool {
    name.contains(".")
}

pub(crate) struct ApiReducer {
    type_aliases: HashMap<String, Vec<String>>,
}

impl ApiReducer {
    pub(crate) fn build(api: &API) -> API {
        let reducer = Self::new(api);
        let mut reduced = api.clone();
        reducer.reduce(&mut reduced);
        reduced
    }

    fn new(api: &API) -> Self {
        let api = api.clone();
        let mut type_aliases = HashMap::<String, Vec<String>>::new();
        let mut add_type_alias = |name: &str, full_name: String| {
            if let Some(existing) = type_aliases.get_mut(name) {
                existing.push(full_name);
            } else {
                type_aliases.insert(name.to_string(), vec![full_name]);
            }
        };

        for module in &api.modules {
            for ty in &module.types {
                let full_name = format!("{}.{}", module.name, ty.name);
                add_type_alias(&full_name, full_name.clone());
                add_type_alias(&ty.name, full_name);
            }
        }

        Self { type_aliases }
    }

    fn reduce(&self, api: &mut API) {
        for module in &mut api.modules {
            let module_name = module.name.clone();
            for ty in &mut module.types {
                self.resolve_refs(&module_name, &mut ty.value);
            }
            for f in &mut module.functions {
                self.reduce_function(&module_name, f);
            }
        }
    }

    fn resolve_refs(&self, module_name: &str, ty: &mut Type) {
        match ty {
            Type::Ref { type_name: name } => {
                if !is_full_name(name) {
                    let full = format!("{}{}", module_name, name);
                    if self.type_aliases.contains_key(&full) {
                        *name = full;
                    } else if let Some(names) = self.type_aliases.get(name) {
                        *name = names[0].clone()
                    }
                }
            }
            Type::Generic { args, .. } => {
                for ty in args {
                    self.resolve_refs(module_name, ty);
                }
            }
            Type::Array { items } => self.resolve_refs(module_name, items),
            Type::EnumOfTypes { types } => {
                for ty in types {
                    self.resolve_refs(module_name, &mut ty.value);
                }
            }
            Type::Optional { inner } => self.resolve_refs(module_name, inner),
            Type::Struct { fields } => {
                for field in fields {
                    self.resolve_refs(module_name, &mut field.value);
                }
            }
            _ => {}
        }
    }

    fn reduce_function(&self, module_name: &str, function: &mut Function) {
        function.params = function
            .params
            .iter()
            .find(|x| x.name == "params")
            .map(|x| vec![x.clone()])
            .unwrap_or(vec![]);
        if function.params.len() > 0 {
            self.resolve_refs(module_name, &mut function.params[0].value);
        }
        match &function.result {
            Type::Generic { type_name: name, args } if name == "ClientResult" => {
                function.result = args[0].clone()
            }
            _ => (),
        };
        self.resolve_refs(module_name, &mut function.result);
    }
}
