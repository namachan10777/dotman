use yaml_rust::{yaml::Hash, Yaml};

use std::env;

use crate::util::resolve_liquid_template;

struct EnvTask {
    envs: Vec<(String, Option<String>)>,
}

impl crate::Task for EnvTask {
    fn name(&self) -> String {
        let envs = self
            .envs
            .iter()
            .take(3)
            .map(|(name, val)| match val {
                Some(s) => format!("{} <= {}", name, s),
                None => format!("remove({})", name),
            })
            .collect::<Vec<_>>()
            .join(", ");
        if self.envs.len() > 3 {
            format!("envs {}...", envs)
        } else {
            format!("envs {}", envs)
        }
    }

    fn execute(&self, _: &crate::TaskContext) -> crate::TaskResult {
        let mut changed = false;
        for (name, value) in &self.envs {
            if let Some(value) = value {
                let value = resolve_liquid_template(value).map_err(|_| {
                    crate::TaskError::WellKnown(format!("cannot resolve env value {}", value))
                })?;
                match env::var(&name) {
                    Ok(s) => {
                        changed |= s != value;
                    }
                    Err(env::VarError::NotPresent) => {
                        changed = true;
                    }
                    Err(env::VarError::NotUnicode(_)) => {
                        changed = true;
                    }
                }
                env::set_var(name, value);
            } else if Err(env::VarError::NotPresent) != env::var(&name) {
                env::remove_var(name);
                changed = true;
            }
        }
        Ok(changed)
    }
}

fn yaml_to_str(yaml: &Yaml) -> Result<Option<String>, crate::Error> {
    match &yaml {
        Yaml::String(s) => Ok(Some(s.clone())),
        Yaml::Real(r) => Ok(Some(r.clone())),
        Yaml::Integer(i) => Ok(Some(i.to_string())),
        Yaml::Boolean(b) => Ok(Some(b.to_string())),
        Yaml::Null => Ok(None),
        _ => Err(crate::Error::InvalidPlaybook(
            "cannot interpret as string".to_owned(),
            yaml.to_owned(),
        )),
    }
}

pub fn parse(obj: &Hash) -> Result<Box<dyn crate::Task>, crate::Error> {
    let envs = obj
        .get(&Yaml::String("envs".to_owned()))
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("env must have \"envs\"".to_owned()))?
        .as_hash()
        .ok_or_else(|| {
            crate::Error::PlaybookLoadFailed("env.envs must be pairs of environments".to_owned())
        })?
        .into_iter()
        .map(|(name, val)| {
            let name = yaml_to_str(name)?.unwrap_or_else(|| "null".to_owned());
            let val = yaml_to_str(val)?;
            Ok((name, val))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Box::new(EnvTask { envs }))
}
