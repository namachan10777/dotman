use regex::Regex;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;
use std::{collections::HashMap, path::Path};
use std::{fmt, fs};
use termion::color;
use yaml_rust::YamlLoader;

pub mod ast;
pub mod tasks;
pub mod util;

use thiserror::Error;

/// The trait of Task
pub trait Task {
    /// return human-readable identity name.
    fn name(&self) -> String;
    /// execute with context.
    fn execute(&self, ctx: &TaskContext) -> TaskResult;
}

#[derive(Debug, Clone)]
enum TargetMatcher {
    HostName(String, Regex),
    Root(bool),
}

impl PartialEq for TargetMatcher {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HostName(x, _), Self::HostName(y, _)) => x == y,
            (Self::Root(x), Self::Root(y)) => x == y,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Scenario {
    name: String,
    tasks: Vec<String>,
    matches: Vec<TargetMatcher>,
}

pub type TaskGroups = HashMap<String, Vec<(String, Box<dyn Task>)>>;
pub type ScheduledTasks<'a> = Vec<(&'a str, &'a [(String, Box<dyn Task>)])>;

/// Compiled configuration
pub struct PlayBook {
    taskgroups: TaskGroups,
    base: PathBuf,
    task_ids: Vec<String>,
    scenarios: Vec<Scenario>,
}

impl fmt::Debug for PlayBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayBook")
            .field(
                "taskgroups",
                &self
                    .taskgroups
                    .iter()
                    .map(|(key, tasks)| {
                        (
                            key,
                            tasks
                                .iter()
                                .map(|(_, task)| task.name())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<HashMap<_, _>>(),
            )
            .field("scenarios", &self.scenarios)
            .finish()
    }
}

/// execution context to pass to Task
#[derive(Debug)]
pub struct TaskContext {
    /// Base directory to execute deploy
    pub base: PathBuf,
    /// Dry-run flag 
    pub dryrun: bool,
    /// Selected deploy scenario
    pub scenario: String,
    /// Cache shared between same task type
    pub cache: Rc<RefCell<Option<Box<dyn Any>>>>,
}

/// Critical errors
#[derive(Debug, Clone)]
pub enum Error {
    /// Found unrecognized member of configuration
    UnrecognizedMembers {
        /// prefix to identity where the error was reported
        prefix: Option<String>,
        /// unrecognized members
        members: Vec<(String, ast::Value)>,
    },
    /// Failed to load playbook
    PlaybookLoadFailed(String),
    /// Failed to load playbook with a part of configuration
    InvalidPlaybook(String, ast::Value),
    /// Taskgroup was not found
    TaskGroupNotFound(String),
    /// No scenario
    AnyScenarioDoesNotMatch,
    /// Failed to resolve template variable
    CannotResolveVar(String, std::env::VarError),
    /// Failed to collect node information
    CannotCollectNodeInformation(String),
}

type TaskResult = Result<bool, TaskError>;
#[derive(Error, Debug)]
/// Error for tasks
pub enum TaskError {
    /// Wellknown error to be reported
    #[error("wellknown {0}")]
    WellKnown(String),
    /// Unknown error that can be considered bugs
    #[error("unknown {0}")]
    Unknown(anyhow::Error),
}

impl From<anyhow::Error> for TaskError {
    fn from(e: anyhow::Error) -> Self {
        TaskError::Unknown(e)
    }
}

fn parse_task(
    taskbuilders: &TaskBuilders,
    yaml: &ast::Value,
) -> Result<(String, Box<dyn Task>), Error> {
    let obj = yaml
        .as_hash()
        .ok_or_else(|| Error::InvalidPlaybook("task must be hash".to_owned(), yaml.to_owned()))?;
    if let Some(ast::Value::Str(key)) = obj.get("type") {
        if let Some(parse) = taskbuilders.get(key.as_str()) {
            let task = parse(obj)?;
            Ok((key.to_owned(), task))
        } else {
            Err(Error::InvalidPlaybook(
                format!("unsupported task \"{}\"", key.as_str()),
                yaml.to_owned(),
            ))
        }
    } else {
        Err(Error::InvalidPlaybook(
            "task must have \"type\" property".to_owned(),
            yaml.to_owned(),
        ))
    }
}

fn parse_taskgroups(yaml: &ast::Value, taskbuilders: &TaskBuilders) -> Result<TaskGroups, Error> {
    yaml.as_hash()
        .ok_or_else(|| {
            Error::InvalidPlaybook("taskgroups must be hash".to_owned(), yaml.to_owned())
        })?
        .iter()
        .map(|(name, tasks)| match (name, tasks) {
            (name, ast::Value::Array(tasks)) => Ok((
                name.to_owned(),
                tasks
                    .iter()
                    .map(|src| parse_task(taskbuilders, src))
                    .collect::<Result<Vec<_>, Error>>()?,
            )),
            _ => Err(Error::InvalidPlaybook(
                "children of taskgropus must be [string]: <task>[]".to_owned(),
                yaml.to_owned(),
            )),
        })
        .collect::<Result<HashMap<_, _>, Error>>()
}

fn parse_matcher(yaml: &ast::Value) -> Result<TargetMatcher, Error> {
    let obj = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("matcher must be hash".to_owned(), yaml.to_owned())
    })?;
    if let Some((target, val)) = obj.iter().next() {
        match target.as_str() {
            "hostname" => {
                ast::verify_hash(obj, &["hostname"], Some("matcher"))?;
                let hostname_re_src = val.as_str().ok_or_else(|| {
                    Error::InvalidPlaybook(
                        "matcher.hostname must be string".to_owned(),
                        val.to_owned(),
                    )
                })?;
                let hostname_regex = regex::Regex::new(hostname_re_src).map_err(|e| {
                    Error::InvalidPlaybook(
                        format!(
                            "cannot compile matcher.hostname {} due to {:?}",
                            hostname_re_src, e
                        ),
                        val.to_owned(),
                    )
                })?;
                Ok(TargetMatcher::HostName(
                    hostname_re_src.to_owned(),
                    hostname_regex,
                ))
            }
            "root" => {
                ast::verify_hash(obj, &["root"], Some("matcher"))?;
                Ok(TargetMatcher::Root(val.as_bool().ok_or_else(|| {
                    Error::InvalidPlaybook(
                        "matcher.root must be boolean".to_owned(),
                        yaml.to_owned(),
                    )
                })?))
            }
            matcher_name => Err(Error::InvalidPlaybook(
                format!("unsupported matcher \"{}\"", matcher_name),
                yaml.to_owned(),
            )),
        }
    } else {
        Err(Error::InvalidPlaybook(
            "matcher must be [string]: <matcher>".to_owned(),
            yaml.to_owned(),
        ))
    }
}

fn parse_scenario(yaml: &ast::Value) -> Result<Scenario, Error> {
    let obj = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("scenario mast be hash".to_owned(), yaml.to_owned())
    })?;

    ast::verify_hash(obj, &["name", "match", "tasks"], Some("scenario"))?;
    if let (
        Some(ast::Value::Str(name)),
        Some(ast::Value::Array(matchers)),
        Some(ast::Value::Array(tasks)),
    ) = (obj.get("name"), obj.get("match"), obj.get("tasks"))
    {
        let mut matches = matchers
            .iter()
            .map(parse_matcher)
            .collect::<Result<Vec<TargetMatcher>, Error>>()?;
        if !matches
            .iter()
            .any(|matcher| matches!(matcher, TargetMatcher::Root(_)))
        {
            matches.push(TargetMatcher::Root(false));
        }
        let tasks = tasks
            .iter()
            .map(|taskname| {
                taskname.as_str().map(|s| s.to_owned()).ok_or_else(|| {
                    Error::InvalidPlaybook(
                        "scenario.tasks must be array of string".to_owned(),
                        yaml.to_owned(),
                    )
                })
            })
            .collect::<Result<Vec<String>, Error>>()?;
        Ok(Scenario {
            tasks,
            matches,
            name: name.to_owned(),
        })
    } else {
        Err(Error::InvalidPlaybook(
            "scenario.name must be string, scenario.match and scenario.tasks must be array"
                .to_owned(),
            yaml.to_owned(),
        ))
    }
}

#[cfg(test)]
mod test_parsers {
    use super::*;

    #[test]
    fn test_parse_scenario() {
        let src = concat!(
            "---\n",
            "name: test_scenario\n",
            "match:\n",
            "- hostname: hoge\n",
            "tasks:\n",
            "- task1\n"
        );
        let yaml = YamlLoader::load_from_str(src).unwrap();
        let ast = ast::Value::from_yaml(yaml[0].clone()).unwrap();
        assert_eq!(
            parse_scenario(&ast).unwrap(),
            Scenario {
                tasks: vec!["task1".to_owned()],
                matches: vec![
                    TargetMatcher::HostName("hoge".to_owned(), Regex::new(r"hoge").unwrap()),
                    TargetMatcher::Root(false)
                ],
                name: "test_scenario".to_owned()
            }
        );
    }
}

struct NodeInformation {
    root: bool,
    hostname: OsString,
}

impl NodeInformation {
    fn collect() -> anyhow::Result<Self> {
        Ok(Self {
            #[cfg(target_family = "unix")]
            root: unsafe { libc::getuid() == 0 },
            #[cfg(target_family = "windows")]
            root: false,
            hostname: hostname::get()?,
        })
    }
}

fn match_scenario<'a>(
    scenarios: &'a [Scenario],
    node_info: &NodeInformation,
) -> Option<&'a Scenario> {
    for scenario in scenarios {
        if scenario.matches.iter().all(|matcher| match matcher {
            TargetMatcher::HostName(_, hostname_re) => {
                hostname_re.is_match(&node_info.hostname.to_string_lossy())
            }
            TargetMatcher::Root(is_root) => *is_root == node_info.root,
        }) {
            return Some(scenario);
        }
    }
    None
}

#[cfg(test)]
mod test_matcher {
    use super::*;

    #[test]
    fn test_match_scenario() {
        let src_nonroot1 = concat!(
            "---\n",
            "name: test_scenario\n",
            "match:\n",
            "- hostname: ^hoge$\n",
            "tasks:\n",
            "- task1\n"
        );
        let src_nonroot2 = concat!(
            "---\n",
            "name: test_scenario\n",
            "match:\n",
            "- hostname: ^fuga$\n",
            "tasks:\n",
            "- task1\n"
        );
        let src_root = concat!(
            "---\n",
            "name: test_scenario\n",
            "match:\n",
            "- hostname: ^hoge$\n",
            "- root: true\n",
            "tasks:\n",
            "- task1\n"
        );
        let nonroot1 = parse_scenario(
            &ast::Value::from_yaml(
                YamlLoader::load_from_str(src_nonroot1)
                    .unwrap()
                    .into_iter()
                    .next()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let nonroot2 = parse_scenario(
            &ast::Value::from_yaml(
                YamlLoader::load_from_str(src_nonroot2)
                    .unwrap()
                    .into_iter()
                    .next()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let root = parse_scenario(
            &ast::Value::from_yaml(
                YamlLoader::load_from_str(src_root)
                    .unwrap()
                    .into_iter()
                    .next()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let scenarios = vec![nonroot1.clone(), nonroot2.clone(), root.clone()];
        assert_eq!(
            match_scenario(
                scenarios.as_slice(),
                &NodeInformation {
                    hostname: OsString::from("hoge".to_owned()),
                    root: false,
                }
            ),
            Some(&nonroot1)
        );
        assert_eq!(
            match_scenario(
                scenarios.as_slice(),
                &NodeInformation {
                    hostname: OsString::from("fuga".to_owned()),
                    root: false,
                }
            ),
            Some(&nonroot2)
        );
        assert_eq!(
            match_scenario(
                scenarios.as_slice(),
                &NodeInformation {
                    hostname: OsString::from("hoge".to_owned()),
                    root: true,
                }
            ),
            Some(&root)
        );
        assert_eq!(
            match_scenario(
                scenarios.as_slice(),
                &NodeInformation {
                    hostname: OsString::from("bar".to_owned()),
                    root: true,
                }
            ),
            None
        );
    }
}

fn enlist_taskgroups<'a>(
    taskgroups: &'a TaskGroups,
    taskgroup_names: &'a [String],
) -> Result<ScheduledTasks<'a>, Error> {
    taskgroup_names
        .iter()
        .map(|taskgroup_name| {
            taskgroups
                .get(taskgroup_name)
                .map(|task| (taskgroup_name.as_str(), task.as_slice()))
                .ok_or_else(|| Error::TaskGroupNotFound(taskgroup_name.to_owned()))
        })
        .collect::<Result<Vec<_>, Error>>()
}

pub type Stats = Vec<(String, Vec<(String, TaskResult)>)>;

pub type TaskBuilder = Box<dyn Fn(&HashMap<String, ast::Value>) -> Result<Box<dyn Task>, Error>>;
pub type TaskBuilders = HashMap<String, TaskBuilder>;

impl PlayBook {
    /// Load configuration from yaml text with taskbuilders.
    pub fn load_config(config: &str, taskbuilders: TaskBuilders) -> Result<Self, Error> {
        let playbook_src = fs::read_to_string(Path::new(&config)).map_err(|e| {
            Error::PlaybookLoadFailed(format!("cannot read playbook {} due to {:?}", config, e))
        })?;
        let playbook_ast = ast::Value::from_yaml(
            YamlLoader::load_from_str(&playbook_src)
                .map_err(|_| {
                    Error::PlaybookLoadFailed(format!("playbook {} has invalid syntax", config))
                })?
                .get(0)
                .ok_or_else(|| Error::PlaybookLoadFailed(format!("playbook {} is empty", config)))?
                .clone(),
        )
        .map_err(|_| Error::PlaybookLoadFailed("cannot convert to general ast".to_owned()))?
        .as_hash()
        .ok_or_else(|| Error::PlaybookLoadFailed("invalid playbook".to_owned()))?
        .clone();
        ast::verify_hash(&playbook_ast, &["taskgroups", "scenarios"], None)?;
        let taskgroups = playbook_ast
            .get("taskgroups")
            .ok_or_else(|| Error::PlaybookLoadFailed("taskgroups is not found".to_owned()))?;
        let scenarios = playbook_ast
            .get("scenarios")
            .ok_or_else(|| Error::PlaybookLoadFailed("scenarios is not found".to_owned()))?;
        let taskgroups = parse_taskgroups(taskgroups, &taskbuilders)?;
        let scenarios = scenarios
            .as_array()
            .ok_or_else(|| {
                Error::InvalidPlaybook("scenario must be array".to_owned(), scenarios.to_owned())
            })?
            .iter()
            .map(parse_scenario)
            .collect::<Result<Vec<Scenario>, Error>>()?;
        Ok(PlayBook {
            taskgroups,
            task_ids: taskbuilders
                .into_iter()
                .map(|(task, _)| task)
                .collect::<Vec<_>>(),
            base: Path::new(config)
                .parent()
                .ok_or_else(|| {
                    Error::PlaybookLoadFailed(format!("cannot take parent of {}", config))
                })?
                .to_owned(),
            scenarios,
        })
    }

    /// Enlist selected tasks by scenario.
    pub fn deploys(&self, scenario: Option<&str>) -> Result<(String, ScheduledTasks), Error> {
        let scenario = if let Some(scenario) = scenario {
            self.scenarios
                .iter()
                .find(|s| s.name == scenario)
                .ok_or(Error::AnyScenarioDoesNotMatch)?
        } else {
            match_scenario(
                &self.scenarios,
                &NodeInformation::collect()
                    .map_err(|e| Error::CannotCollectNodeInformation(format!("{:?}", e)))?,
            )
            .ok_or(Error::AnyScenarioDoesNotMatch)?
        };
        Ok((
            scenario.name.to_owned(),
            enlist_taskgroups(&self.taskgroups, scenario.tasks.as_slice())?,
        ))
    }

    /// Utility to execute playbook graphicaly
    pub fn execute_graphicaly(&self, dryrun: bool, scenario: Option<&str>) -> Result<(), Error> {
        let (scenario, taskgroups) = self.deploys(scenario)?;
        let mut caches = HashMap::new();
        for task in &self.task_ids {
            caches.insert(task, Rc::new(RefCell::new(None)));
        }

        for (group, tasks) in taskgroups {
            println!("[{}]", group);
            for (id, task) in tasks {
                let task_name = task.name();
                let ctx = TaskContext {
                    dryrun,
                    scenario: scenario.clone(),
                    base: self.base.clone(),
                    cache: caches.get(&id).expect("already registered").clone(),
                };
                let result = task.execute(&ctx);
                match result {
                    Ok(true) => println!(
                        "{}[Changed] {}{}",
                        color::Fg(color::Yellow),
                        color::Fg(color::White),
                        task_name
                    ),
                    Ok(false) => println!(
                        "{}[Ok]      {}{}",
                        color::Fg(color::Green),
                        color::Fg(color::LightWhite),
                        task_name
                    ),
                    Err(TaskError::WellKnown(msg)) => {
                        println!(
                            "{}[Failed]  {}{}",
                            color::Fg(color::Red),
                            color::Fg(color::Reset),
                            task_name
                        );
                        println!("  -> {}", msg);
                    }
                    Err(TaskError::Unknown(e)) => {
                        println!(
                            "{}[Failed]  {}{}",
                            color::Fg(color::Red),
                            color::Fg(color::Reset),
                            task_name
                        );
                        println!("  -> {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}
