use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};

use sha2::{Digest, Sha256};

struct WgetTask {
    sha256: String,
    dest: String,
    url: String,
}

fn check_sha256(sha256: &str, data: &[u8]) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize()) == sha256
}

impl crate::Task for WgetTask {
    fn name(&self) -> String {
        format!("wget {}", self.url)
    }

    fn execute(&self, _: &crate::TaskContext) -> crate::TaskResult {
        let mut buf = Vec::new();
        if let Ok(mut f) = fs::File::open(&self.dest) {
            if f.read_to_end(&mut buf).is_ok() {
                if check_sha256(&self.sha256, buf.as_slice()) {
                    return Ok(false);
                }
            }
        }
        let mut res = reqwest::blocking::get(&self.url).map_err(|e| {
            crate::TaskError::WellKnown(format!("cannot download {} due to {:?}", &self.url, e))
        })?;
        buf.clear();
        res.read_to_end(&mut buf).map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot read response body of {} due to {:?}",
                &self.url, e
            ))
        })?;

        if !check_sha256(&self.sha256, buf.as_slice()) {
            return Err(crate::TaskError::WellKnown(
                "inconsistent hash value of downloaded file".to_owned(),
            ));
        }

        let f = fs::File::create(&self.dest).map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot open {} as writ-mode due to {:?}",
                &self.dest, e
            ))
        })?;
        let mut writer = io::BufWriter::new(f);
        writer.write_all(buf.as_slice()).map_err(|e| {
            crate::TaskError::WellKnown(format!(
                "cannot write response body to local file due to {:?}",
                e
            ))
        })?;
        Ok(true)
    }
}

pub fn parse(
    obj: &HashMap<String, crate::ast::Value>,
) -> Result<Box<dyn crate::Task>, crate::Error> {
    let sha256 = obj
        .get("sha256")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.sha256 is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.sha256 must be string".to_owned()))?
        .to_owned();
    let url = obj
        .get("url")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.sha256 is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.url must be string".to_owned()))?
        .to_owned();
    let dest = obj
        .get("dest")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.sha256 is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("wget.dest must be string".to_owned()))?
        .to_owned();
    Ok(Box::new(WgetTask { sha256, url, dest }))
}
