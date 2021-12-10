//! Builtin env task.
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::{fs, io, os};

/// Implementation of [Task trait](../../trait.Task.html).
pub struct LinkTask {
    src: String,
    dest: String,
}

#[cfg(target_family = "unix")]
fn symlink(src: &Path, dest: &Path) -> io::Result<()> {
    os::unix::fs::symlink(src, dest)
}

#[cfg(target_family = "windows")]
fn symlink(src: &Path, dest: &Path) -> io::Result<()> {
    if fs::metadata(src)?.is_dir() {
        os::windows::fs::symlink_dir(src, dest)
    } else {
        os::windows::fs::symlink_file(src, dest)
    }
}

#[async_trait::async_trait]
impl crate::Task for LinkTask {
    fn name(&self) -> String {
        format!("link {} => {}", self.src, self.dest)
    }

    async fn execute(&self, _: &crate::TaskContext) -> crate::TaskResult {
        let src = crate::util::resolve_liquid_template(&self.src).map_err(|e| {
            crate::TaskError::WellKnown(format!("cannot resolve tasks.link.src due to {:?}", e))
        })?;
        let dest = crate::util::resolve_liquid_template(&self.dest).map_err(|e| {
            crate::TaskError::WellKnown(format!("cannot resolve tasks.link.dest due to {:?}", e))
        })?;
        let src = OsStr::new(&src);
        let dest = OsStr::new(&dest);
        let src = Path::new(src).canonicalize().map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot canonicalize tasks.link.dest due to {:?}",
                e
            ))
        })?;
        let dest = Path::new(dest);

        if fs::read_link(&dest).map(|p| p == src).unwrap_or(false) {
            return Ok(false);
        }
        if let Ok(meta) = fs::metadata(&dest) {
            // TODO: use is_link
            if meta.is_dir() {
                fs::remove_dir_all(&dest).map_err(|e| {
                    crate::TaskError::WellKnown(format!(
                        "cannot remove dir {:?} due to {:?}",
                        dest, e
                    ))
                })?;
            } else {
                fs::remove_file(&dest).map_err(|e| {
                    crate::TaskError::WellKnown(format!(
                        "cannot remove dir {:?} due to {:?}",
                        dest, e
                    ))
                })?;
            }
        }
        fs::create_dir_all(dest.parent().ok_or_else(|| {
            crate::TaskError::WellKnown(format!(
                "cannot take parent of desitination path {:?}",
                dest
            ))
        })?)
        .map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot create parent dir of desitination path {:?} due to {:?}",
                dest, e
            ))
        })?;
        symlink(&src, dest).map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot link {:?} to {:?} due to {:?}",
                src, dest, e
            ))
        })?;
        Ok(true)
    }
}

/// parse task as a link task
pub fn parse(obj: &HashMap<String, crate::ast::Value>) -> Result<crate::TaskEntity, crate::Error> {
    crate::ast::verify_hash(obj, &["type", "src", "dest"], Some("tasks.link"))?;
    let src = obj
        .get("src")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("link.src is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("link.src must be string".to_owned()))?
        .to_owned();
    let dest = obj
        .get("dest")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("link.dest is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("link.dest must be string".to_owned()))?
        .to_owned();
    Ok(crate::TaskEntity::Link(LinkTask { src, dest }))
}
