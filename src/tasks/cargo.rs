use crate::TaskError;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, char, digit1},
    combinator::{recognize, verify},
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};
use std::collections::HashMap;
use yaml_rust::{yaml::Hash, Yaml};

type Package = (String, String);
type Packages = HashMap<String, String>;

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
    let (src, _) = tag(":\n")(src)?;
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

pub struct CargoTask {
    package: String,
    version: Option<String>,
}

impl crate::Task for CargoTask {
    fn name(&self) -> String {
        if let Some(version) = &self.version {
            format!("cargo install {}:{}", self.package, version)
        } else {
            format!("cargo install {}", self.package)
        }
    }

    fn execute(&self, _: &crate::TaskContext) -> crate::TaskResult {
        if let Ok(output) = duct::cmd("cargo", &["install", "--list"]).read() {
            if let Ok((_, packages)) = parse_cargo_install_list(&output) {
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
            } else {
                Err(TaskError::WellKnown(
                    "cannot parse installed cargo package information. this is bug".to_owned(),
                ))
            }
        } else {
            Err(TaskError::WellKnown(
                "cannot fetch installed cargo package information".to_owned(),
            ))
        }
    }
}

pub fn parse(obj: &Hash) -> Result<Box<dyn crate::Task>, crate::Error> {
    let package = obj
        .get(&Yaml::String("package".to_owned()))
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cargo.package is required".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cargo.package must be string".to_owned()))?
        .to_owned();
    let version = obj.get(&Yaml::String("version".to_owned())).map(|version| {
        version.as_str().ok_or_else(|| {
            crate::Error::InvalidPlaybook(
                "cargo.version must be string".to_owned(),
                version.to_owned(),
            )
        })
    });
    if let Some(version) = version {
        Ok(Box::new(CargoTask {
            package,
            version: Some(version?.to_owned()),
        }))
    } else {
        Ok(Box::new(CargoTask {
            package,
            version: None,
        }))
    }
}
