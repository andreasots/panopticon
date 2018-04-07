use std::collections::HashMap;
use tera::{GlobalFn, Value};

include!(concat!(env!("OUT_DIR"), "/webpack_manifest.rs"));

pub fn make_webpacked() -> GlobalFn {
    Box::new(move |args: HashMap<String, Value>| {
        args.get("name")
            .and_then(Value::as_str)
            .map(|name| Value::String(MANIFEST[name].to_string()))
            .ok_or_else(|| "required argument `name` missing".into())
    })
}
