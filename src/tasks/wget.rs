//! Builtin wget task.
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{fs, io};

use sha2::{Digest, Sha256};

use crate::util::resolve_liquid_template;
use crate::TaskEntity;

enum Sha256Set {
    Each(HashMap<String, String>),
    All(String),
}

impl Sha256Set {
    fn get(&self, key: &str) -> Option<&String> {
        match self {
            Sha256Set::All(s) => Some(s),
            Sha256Set::Each(hash) => hash.get(key),
        }
    }
}

/// Implementation of [Task trait](../../trait.Task.html).
pub struct WgetTask {
    sha256: Sha256Set,
    dest: String,
    url: String,
}

fn check_sha256(sha256: &str, data: &[u8]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize()) == sha256
}

#[async_trait::async_trait]
impl crate::Task for WgetTask {
    fn name(&self) -> String {
        format!("wget {}", self.url)
    }

    async fn execute(&self, ctx: &crate::TaskContext) -> crate::TaskResult {
        let dest = resolve_liquid_template(&self.dest).map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot resolve template {} due to {:?}",
                &self.dest, e
            ))
        })?;
        let mut buf = Vec::new();
        let sha256 = &self.sha256.get(&ctx.scenario).ok_or_else(|| {
            crate::TaskError::WellKnown(format!("wget.sha256.{} is not found", &ctx.scenario))
        })?;
        if let Ok(mut f) = fs::File::open(&dest).await {
            if f.read_to_end(&mut buf).await.is_ok() && check_sha256(sha256, buf.as_slice()) {
                return Ok(false);
            }
        }
        let res = reqwest::get(&self.url).await.map_err(|e| {
            crate::TaskError::WellKnown(format!("cannot download {} due to {:?}", &self.url, e))
        })?;
        let buf = res.bytes().await.map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot read response body of {} due to {:?}",
                &self.url, e
            ))
        })?;

        if !check_sha256(sha256, buf.as_ref()) {
            return Err(crate::TaskError::WellKnown(
                "inconsistent hash value of downloaded file".to_owned(),
            ));
        }

        let f = fs::File::create(&dest).await.map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot open {} as writ-mode due to {:?}",
                &dest, e
            ))
        })?;
        let mut writer = io::BufWriter::new(f);
        writer.write_all(buf.as_ref()).await.map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot write response body to local file due to {:?}",
                e
            ))
        })?;
        Ok(true)
    }
}

/// parse task as a wget task
pub fn parse(obj: &HashMap<String, crate::ast::Value>) -> Result<crate::TaskEntity, crate::Error> {
    crate::ast::verify_hash(obj, &["type", "url", "dest", "sha256"], Some("tasks.wget"))?;
    let sha256 = obj
        .get("sha256")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.sha256 is required".to_owned()))?;
    let sha256 = match sha256 {
        crate::ast::Value::Hash(hash) => Ok(Sha256Set::Each(
            hash.iter()
                .map(|(key, value)| {
                    value
                        .as_str()
                        .ok_or_else(|| {
                            crate::Error::PlaybookLoadFailed(
                                "wget.sha256.xxx must be string".to_owned(),
                            )
                        })
                        .map(|v| (key.to_owned(), v.to_owned()))
                })
                .collect::<Result<HashMap<_, _>, _>>()?,
        )),
        crate::ast::Value::Str(s) => Ok(Sha256Set::All(s.to_owned())),
        _ => Err(crate::Error::PlaybookLoadFailed(
            "wget.sha256 must be hash or string".to_owned(),
        )),
    }?;
    let url = obj
        .get("url")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.url is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.url must be string".to_owned()))?
        .to_owned();
    let dest = obj
        .get("dest")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.dest is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.dest must be string".to_owned()))?
        .to_owned();
    Ok(TaskEntity::Wget(WgetTask { sha256, url, dest }))
}
