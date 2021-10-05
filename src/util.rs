use std::env;
use std::path::{self, Path, PathBuf};

pub fn resolve_desitination_path(path: &str) -> Result<PathBuf, crate::Error> {
    Ok(Path::new(
        &path
            .split(path::MAIN_SEPARATOR)
            .map(|elem| {
                if let Some(var_name) = elem.strip_prefix('$') {
                    env::var(var_name).map_err(|_| crate::Error::CannotResolveVar(elem.to_owned()))
                } else if elem.starts_with("\\$") {
                    Ok(elem[1..].to_owned())
                } else {
                    Ok(elem.to_owned())
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(&path::MAIN_SEPARATOR.to_string()),
    )
    .to_owned())
}
