cfg_if::cfg_if! {
    if #[cfg(feature = "tera")] {
// CFG IF

use std::{collections::BTreeMap, sync::RwLock};

use lazy_static::lazy_static;
use crate::StripPath;

use super::{TemplateEngine, TreeToTemplateContext, Result};

lazy_static! {
    pub static ref TERA: RwLock<Option<(tera::Tera, BTreeMap<String, serde_json::Value>)>> =
        RwLock::new(None);
}

pub struct Tera;
impl TemplateEngine for Tera {
    fn context() -> BTreeMap<String, serde_json::Value> {
        BTreeMap::new()
    }

    fn parse_path(path: &str) -> String {
        path.to_string()
    }

    fn init<T: Into<String>>(path: T, globals: BTreeMap<String, serde_json::Value>) {
        let is_some = TERA.read().unwrap().is_some();
        let path: String = path.into();
        if !is_some {
            match tera::Tera::new(&format!("{}/**/*", path.norm_strip_slashes())) {
                Ok(t) => *TERA.write().unwrap() = Some((t, globals)),
                Err(err) => panic!("Failed to initialize Tera templating engine: {}", err),
            }
        }
    }

    fn globals() -> BTreeMap<String, serde_json::Value> {
        let tera = TERA.read().unwrap();
        match &(*tera) {
            Some(tera) => tera.1.clone(),
            None => Tera::context(),
        }
    }

    fn render(path: String, context: BTreeMap<String, serde_json::Value>) -> Result<String> {
        let tera = TERA.read().unwrap();
        match &(*tera) {
            Some(tera) => tera
                .0
                .render(&path, &Tera::to_context(context))
                .map_err(|err| (500, err.to_string())),
            None => Err((500, "Tera templating engine is not active".to_string())),
        }
    }
}

impl TreeToTemplateContext for Tera {
    type Return = tera::Context;
    fn to_context(map: BTreeMap<String, serde_json::Value>) -> Self::Return {
        let mut context = tera::Context::new();
        for (key, value) in map.iter() {
            context.insert(key, &value);
        }
        context
    }
}

// CFG END IF
    }
}
