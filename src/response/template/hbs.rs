cfg_if::cfg_if! {
    if #[cfg(feature = "handlebars")] {
// CFG IF

use std::{collections::BTreeMap, ffi::OsStr, path::Path, sync::RwLock};

use lazy_static::lazy_static;
use crate::StripPath;

use super::{TemplateEngine, TreeToTemplateContext, Result};

lazy_static! {
    pub static ref HANDLEBARS: RwLock<
        Option<(
            handlebars::Handlebars<'static>,
            BTreeMap<String, serde_json::Value>
        )>,
    > = RwLock::new(None);
}

pub struct Handlebars;
impl TemplateEngine for Handlebars {
    fn context() -> BTreeMap<String, serde_json::Value> {
        BTreeMap::new()
    }

    fn parse_path(path: &str) -> String {
        match Path::new(path).extension().and_then(OsStr::to_str) {
            Some(ext) => path.strip_suffix(&format!(".{}", ext)).unwrap().to_string(),
            None => path.to_string(),
        }
    }

    fn init<T: Into<String>>(path: T, globals: BTreeMap<String, serde_json::Value>) {
        let is_some = HANDLEBARS.read().unwrap().is_some();
        let path: String = path.into();
        if !is_some {
            let mut engine = handlebars::Handlebars::new();
            match engine
                .register_templates_directory(".hbs", &format!("{}/", path.norm_strip_slashes()))
            {
                Ok(_) => *HANDLEBARS.write().unwrap() = Some((engine, globals)),
                Err(err) => panic!("Failed to initialize Handlebars templating engine: {}", err),
            }
        }
    }

    fn globals() -> BTreeMap<String, serde_json::Value> {
        let hbs = HANDLEBARS.read().unwrap();
        match &(*hbs) {
            Some(hbs) => hbs.1.clone(),
            None => Handlebars::context(),
        }
    }

    fn render(path: String, context: BTreeMap<String, serde_json::Value>) -> Result<String> {
        let hbs = HANDLEBARS.read().unwrap();
        match &(*hbs) {
            Some(hbs) => hbs
                .0
                .render(
                    &path,
                    &serde_json::to_value(context).map_err(|err| {
                        (
                            500,
                            format!("Failed to convert Handlebars context to json: {}", err),
                        )
                    })?,
                )
                .map_err(|err| (500, err.to_string())),
            None => Err((
                500,
                "Handlebars templating engine is not active".to_string(),
            )),
        }
    }
}

impl TreeToTemplateContext for Handlebars {
    type Return = BTreeMap<String, serde_json::Value>;
    fn to_context(map: BTreeMap<String, serde_json::Value>) -> Self::Return {
        map
    }
}

// CFG END IF
    }
}
