use sha2::Digest;
use std::{fs, io::Read};
use yaml_rust::{yaml::Hash, Yaml};

struct ShTask {
    cmd: (String, Vec<String>),
    test: Option<(String, Option<String>)>,
}

impl crate::Task for ShTask {
    fn name(&self) -> String {
        format!("sh \"{} {}\"", self.cmd.0, self.cmd.1.join(" "))
    }

    fn execute(&self, _: &crate::TaskContext) -> crate::TaskResult {
        match &self.test {
            Some((path, Some(sha256))) => {
                let mut buf = Vec::new();
                if let Ok(Ok(_)) = fs::File::open(path).map(|mut f| f.read_to_end(&mut buf)) {
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(&buf);
                    let hashed = hasher.finalize();
                    if &hex::encode(hashed) == sha256 {
                        return Ok(false);
                    }
                }
            }
            Some((path, None)) => {
                if fs::metadata(path).is_ok() {
                    return Ok(false);
                }
            }
            None => (),
        }
        duct::cmd(&self.cmd.0, &self.cmd.1)
            .read()
            .map_err(|e| crate::TaskError::WellKnown(format!("sh error {:?}", e)))?;
        Ok(true)
    }
}

pub fn parse(obj: &Hash) -> Result<Box<dyn crate::Task>, crate::Error> {
    let mut cmd = obj
        .get(&Yaml::String("cmd".to_owned()))
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("sh must have \"cmd\"".to_owned()))?
        .as_vec()
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
    let test = obj.get(&Yaml::String("test".to_owned())).map(|s| {
        s.as_str()
            .ok_or_else(|| crate::Error::PlaybookLoadFailed("sh.test must be string".to_owned()))
    });
    let sha256 = obj.get(&Yaml::String("sha256".to_owned())).map(|s| {
        s.as_str()
            .ok_or_else(|| crate::Error::PlaybookLoadFailed("sh.sha256 must be string".to_owned()))
    });
    let test = match (test, sha256) {
        (Some(test), Some(sha256)) => Some((test?.to_owned(), Some(sha256?.to_owned()))),
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
            obj.get(&Yaml::String("cmd".to_owned()))
                .expect("already suceeded")
                .to_owned(),
        ))
    }
}
