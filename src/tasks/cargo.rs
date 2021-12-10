//! Builtin cargo task.
use crate::TaskError;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, char, digit1},
    combinator::{opt, recognize, verify},
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};
use std::collections::HashMap;

type Package = (String, String);
type Packages = HashMap<String, String>;
pub type Cache = Packages;

fn parse_installed_package(src: &str) -> IResult<&str, Package> {
    let (src, package) = recognize(many1(alt((alphanumeric1, tag("-"), tag("_")))))(src)?;
    let (src, _) = char(' ')(src)?;
    let (src, version) = recognize(tuple((
        char('v'),
        digit1,
        char('.'),
        digit1,
        char('.'),
        digit1,
    )))(src)?;
    let (src, _) = tuple((
        opt(tuple((tag(" "), many1(verify(anychar, |c| *c != ':'))))),
        tag(":\n"),
    ))(src)?;
    let (src, _) = many0(tuple((
        tag("    "),
        many1(verify(anychar, |c| *c != '\n')),
        char('\n'),
    )))(src)?;
    Ok((src, (package.to_owned(), version.to_owned())))
}

fn parse_cargo_install_list(src: &str) -> IResult<&str, Packages> {
    let (src, packages) = many0(parse_installed_package)(src)?;
    Ok((src, packages.into_iter().collect::<HashMap<_, _>>()))
}

#[cfg(test)]
mod test_parser {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn test_parse_cargo_install_list() {
        let src = concat!(
            "bandwhich v0.20.0:\n",
            "    bandwhich\n",
            "bingrep v0.9.0:\n",
            "    bingrep\n",
            "cargo-edit v0.8.0:\n",
            "    cargo-add\n",
            "    cargo-rm\n",
            "    cargo-set-version\n",
            "    cargo-upgrade\n",
            "gping v1.2.5:\n",
            "    gping\n",
            "helix-term v0.1.0 (/home/namachan/Project/github.com/topecongiro/helix/helix-term):\n",
            "    hx\n",
            "zoxide v0.7.5:\n",
            "    zoxide\n",
        );
        assert_eq!(
            parse_cargo_install_list(src),
            Ok((
                "",
                hashmap! {
                    "bandwhich".to_owned() => "v0.20.0".to_owned(),
                    "bingrep".to_owned() => "v0.9.0".to_owned(),
                    "cargo-edit".to_owned() => "v0.8.0".to_owned(),
                    "gping".to_owned() => "v1.2.5".to_owned(),
                    "helix-term".to_owned() => "v0.1.0".to_owned(),
                    "zoxide".to_owned() => "v0.7.5".to_owned(),
                }
            ))
        );
    }
}

/// Implementation of [Task trait](../../trait.Task.html).
pub struct CargoTask {
    package: String,
    version: Option<String>,
}

#[async_trait::async_trait]
impl crate::Task for CargoTask {
    fn name(&self) -> String {
        if let Some(version) = &self.version {
            format!("cargo install {}:{}", self.package, version)
        } else {
            format!("cargo install {}", self.package)
        }
    }

    async fn execute(&self, ctx: &crate::TaskContext) -> crate::TaskResult {
        let packages = if ctx.cache.read().await.is_some() {
            rmp_serde::from_read_ref(&ctx.cache.read().await.as_ref().expect("checked").clone())
                .map_err(|e| TaskError::Unknown(e.into()))?
        } else {
            let output = duct::cmd("cargo", &["install", "--list"])
                .read()
                .map_err(|_| {
                    TaskError::WellKnown(
                        "cannot fetch installed cargo package information".to_owned(),
                    )
                })?;
            let (_, packages) = parse_cargo_install_list(&output).map_err(|_| {
                TaskError::WellKnown(
                    "cannot parse installed cargo package information. this is bug".to_owned(),
                )
            })?;
            *ctx.cache.write().await =
                Some(rmp_serde::to_vec(&packages).map_err(|e| TaskError::Unknown(e.into()))?);
            packages
        };
        match &self.version {
            Some(version) => {
                if packages.get(&self.package) != Some(version) {
                    duct::cmd("cargo", &["install", &self.package, "--version", version])
                        .read()
                        .map_err(|e| {
                            TaskError::WellKnown(format!(
                                "cannot install package {}:{} due to {:?}",
                                self.package, version, e
                            ))
                        })?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => {
                if packages.contains_key(&self.package) {
                    Ok(false)
                } else {
                    duct::cmd("cargo", &["install", &self.package])
                        .read()
                        .map_err(|e| {
                            TaskError::WellKnown(format!(
                                "cannot install package {} due to {:?}",
                                self.package, e
                            ))
                        })?;
                    Ok(true)
                }
            }
        }
    }
}

/// parse task section as a cp task
pub fn parse(obj: &HashMap<String, crate::ast::Value>) -> Result<crate::TaskEntity, crate::Error> {
    crate::ast::verify_hash(obj, &["type", "version", "package"], Some("tasks.cargo"))?;
    let package = obj
        .get("package")
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cargo.package is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cargo.package must be string".to_owned()))?
        .to_owned();
    let version = obj.get("version").map(|version| {
        version.as_str().ok_or_else(|| {
            crate::Error::InvalidPlaybook(
                "cargo.version must be string".to_owned(),
                version.to_owned(),
            )
        })
    });
    if let Some(version) = version {
        Ok(crate::TaskEntity::Cargo(CargoTask {
            package,
            version: Some(version?.to_owned()),
        }))
    } else {
        Ok(crate::TaskEntity::Cargo(CargoTask {
            package,
            version: None,
        }))
    }
}
