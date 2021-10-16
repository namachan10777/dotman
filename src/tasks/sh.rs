//! Builtin sh task.
use sha2::Digest;
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::{fs, io::Read};

use crate::TaskError;

/// Implementation of [Task trait](../../trait.Task.html).
struct ShTask {
    cmd: (String, Vec<String>),
    test: Option<(String, Option<HashMap<String, String>>)>,
}

fn check_sha256(sha: &str, path: &Path) -> io::Result<bool> {
    let mut buf = Vec::new();
    fs::File::open(path).map(|mut f| f.read_to_end(&mut buf))??;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&buf);
    let hashed = hasher.finalize();
    Ok(hex::encode(hashed) == sha)
}

impl crate::Task for ShTask {
    fn name(&self) -> String {
        format!("sh \"{} {}\"", self.cmd.0, self.cmd.1.join(" "))
    }

    fn execute(&self, ctx: &crate::TaskContext) -> crate::TaskResult {
        match &self.test {
            Some((path, Some(sha256))) => {
                let path = crate::util::resolve_liquid_template(path)
                    .map_err(|_| TaskError::WellKnown(format!("cannot resolve path {}", path)))?;
                let sha256 = sha256.get(&ctx.scenario).ok_or_else(|| {
                    crate::TaskError::WellKnown(format!("sh.sha256.{} is not found", &ctx.scenario))
                })?;
                if check_sha256(sha256, Path::new(&path)).unwrap_or(false) {
                    Ok(false)
                } else {
                    duct::cmd(&self.cmd.0, &self.cmd.1)
                        .read()
                        .map_err(|e| crate::TaskError::WellKnown(format!("sh error {:?}", e)))?;
                    if !check_sha256(sha256, Path::new(&path))
                        .map_err(|_| TaskError::WellKnown(format!("cannot hash file {:?}", path)))?
                    {
                        return Err(TaskError::WellKnown(format!(
                            "hash inconsistent {:?}",
                            path
                        )));
                    }
                    Ok(true)
                }
            }
            Some((path, None)) => {
                let path = crate::util::resolve_liquid_template(path)
                    .map_err(|_| TaskError::WellKnown(format!("cannot resolve path {}", path)))?;
                if fs::metadata(&path).is_ok() {
                    return Ok(false);
                }
                duct::cmd(&self.cmd.0, &self.cmd.1)
                    .read()
                    .map_err(|e| crate::TaskError::WellKnown(format!("sh error {:?}", e)))?;
                if fs::metadata(&path).is_ok() {
                    Ok(false)
                } else {
                    return Err(TaskError::WellKnown(format!(
                        "file {:?} isn't created",
                        path
                    )));
                }
            }
            None => {
                duct::cmd(&self.cmd.0, &self.cmd.1)
                    .read()
                    .map_err(|e| crate::TaskError::WellKnown(format!("sh error {:?}", e)))?;
                Ok(true)
            }
        }
    }
}

/// parse task section as a sh task
pub fn parse(
    obj: &HashMap<String, crate::ast::Value>,
) -> Result<Box<dyn crate::Task>, crate::Error> {
    crate::ast::verify_hash(obj, &["type", "cmd", "test", "sha256"], Some("tasks.env"))?;
    let mut cmd = obj
        .get("cmd")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("sh must have \"cmd\"".to_owned()))?
        .as_array()
        .ok_or_else(|| {
            crate::Error::PlaybookLoadFailed("sh.cmd must be array of string".to_owned())
        })?
        .iter()
        .map(|s| {
            Ok(s.as_str()
                .ok_or_else(|| {
                    crate::Error::PlaybookLoadFailed("sh.cmd must be array of string".to_owned())
                })?
                .to_owned())
        })
        .collect::<Result<Vec<String>, _>>()?
        .into_iter();
    let test = obj.get("test").map(|s| {
        s.as_str()
            .ok_or_else(|| crate::Error::PlaybookLoadFailed("sh.test must be string".to_owned()))
    });
    let sha256 = obj.get("sha256").map(|s| {
        s.as_hash()
            .ok_or_else(|| crate::Error::PlaybookLoadFailed("sh.sha256 must be string".to_owned()))?
            .iter()
            .map(|(key, value)| {
                value
                    .as_str()
                    .ok_or_else(|| {
                        crate::Error::PlaybookLoadFailed("sh.sha256.xxx must be string".to_owned())
                    })
                    .map(|v| (key.to_owned(), v.to_owned()))
            })
            .collect::<Result<HashMap<_, _>, _>>()
    });
    let test = match (test, sha256) {
        (Some(test), Some(sha256)) => Some((test?.to_owned(), Some(sha256?))),
        (Some(test), None) => Some((test?.to_owned(), None)),
        (None, None) => None,
        (None, Some(_)) => {
            return Err(crate::Error::PlaybookLoadFailed(
                "sh.sha256 requires sh.test".to_owned(),
            ))
        }
    };
    if let Some(exe) = cmd.next() {
        Ok(Box::new(ShTask {
            cmd: (exe, cmd.collect::<Vec<_>>()),
            test,
        }))
    } else {
        Err(crate::Error::InvalidPlaybook(
            "invalid sh.cmd".to_owned(),
            obj.get("cmd").expect("already suceeded").to_owned(),
        ))
    }
}
