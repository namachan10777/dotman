//! Utilities for implementation of tasks.
use kstring::KString;
use std::env;

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
    #[cfg(target_arch = "x86_64")]
    obj.insert(
        KString::from_static("arch"),
        liquid::model::value!("x86_64"),
    );
    #[cfg(target_arch = "x86")]
    obj.insert(KString::from_static("arch"), liquid::model::value!("x86"));
    #[cfg(target_arch = "aarch64")]
    obj.insert(
        KString::from_static("arch"),
        liquid::model::value!("aarch64"),
    );
    obj
}

pub fn resolve_liquid_template(src: &str) -> Result<String, liquid::Error> {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(src)?;
    template.render(&liquid_object_for_global_resolve())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_resolve_liquid_template() {
        assert_eq!(
            resolve_liquid_template("{{env.HOME}}/.config").unwrap(),
            format!("{}/.config", std::env::var("HOME").unwrap())
        );
    }
}
