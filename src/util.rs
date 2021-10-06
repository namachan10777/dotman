use kstring::KString;
use once_cell::sync::Lazy;
use std::env;

pub static LIQUID_OBJECT_GLOBAL: Lazy<liquid::Object> = Lazy::new(liquid_object_for_global_resolve);

fn liquid_object_for_global_resolve() -> liquid::Object {
    let mut obj = liquid::Object::new();
    let mut env_obj = liquid::Object::new();
    for (name, val) in env::vars() {
        env_obj.insert(
            KString::from_string(name),
            liquid::model::Value::scalar(val),
        );
    }
    obj.insert(
        KString::from_static("env"),
        liquid::model::Value::Object(env_obj),
    );
    #[cfg(target_os = "linux")]
    obj.insert(KString::from_static("os"), liquid::model::value!("Linux"));
    #[cfg(target_os = "macos")]
    obj.insert(KString::from_static("os"), liquid::model::value!("Darwin"));
    obj
}

pub fn resolve_liquid_template(src: &str) -> Result<String, liquid::Error> {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(src)?;
    template.render(&(*LIQUID_OBJECT_GLOBAL))
}
