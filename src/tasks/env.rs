//! Builtin env task.
use std::{collections::HashMap, env};

use crate::util::resolve_liquid_template;

/// Implementation of [Task trait](../../trait.Task.html).
pub struct EnvTask {
    envs: Vec<(String, Option<String>)>,
}

#[async_trait::async_trait]
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

    async fn execute(&self, _: &crate::TaskContext) -> crate::TaskResult {
        let mut changed = false;
        for (name, value) in &self.envs {
            if let Some(value) = value {
                let value = resolve_liquid_template(value).map_err(|_| {
                    crate::TaskError::WellKnown(format!("cannot resolve env value {}", value))
                })?;
                match env::var(name) {
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
            } else if Err(env::VarError::NotPresent) != env::var(name) {
                env::remove_var(name);
                changed = true;
            }
        }
        Ok(changed)
    }
}

fn yaml_to_str(yaml: &crate::ast::Value) -> Result<Option<String>, crate::Error> {
    match &yaml {
        crate::ast::Value::Str(s) => Ok(Some(s.clone())),
        crate::ast::Value::Real(r) => Ok(Some(r.to_string())),
        crate::ast::Value::Int(i) => Ok(Some(i.to_string())),
        crate::ast::Value::Bool(b) => Ok(Some(b.to_string())),
        _ => Err(crate::Error::InvalidPlaybook(
            "cannot interpret as string".to_owned(),
            yaml.to_owned(),
        )),
    }
}

/// parse task section as a cp task
pub fn parse(obj: &HashMap<String, crate::ast::Value>) -> Result<crate::TaskEntity, crate::Error> {
    crate::ast::verify_hash(obj, &["type", "envs"], Some("tasks.env"))?;
    let envs = obj
        .get("envs")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("env must have \"envs\"".to_owned()))?
        .as_hash()
        .ok_or_else(|| {
            crate::Error::PlaybookLoadFailed("env.envs must be pairs of environments".to_owned())
        })?
        .iter()
        .map(|(name, val)| {
            let val = yaml_to_str(val)?;
            Ok((name.clone(), val))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(crate::TaskEntity::Env(EnvTask { envs }))
}
