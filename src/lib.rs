use futures::stream::StreamExt;
use regex::Regex;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::{collections::HashMap, path::Path};
use std::{fmt, fs};
use termion::color;
use tokio::sync::{Mutex, RwLock};
use yaml_rust::YamlLoader;

pub mod ast;
pub mod tasks;
pub mod util;

use thiserror::Error;

#[async_trait::async_trait]
/// The trait of Task
pub trait Task {
    /// return human-readable identity name.
    fn name(&self) -> String;
    /// execute with context.
    async fn execute(&self, ctx: &TaskContext) -> TaskResult;
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

pub enum TaskEntity {
    Cargo(tasks::cargo::CargoTask),
    Cp(tasks::cp::CpTask),
    Env(tasks::env::EnvTask),
    Link(tasks::link::LinkTask),
    Sh(tasks::sh::ShTask),
    Wget(tasks::wget::WgetTask),
    Brew(tasks::brew::BrewTask),
}

#[async_trait::async_trait]
impl Task for TaskEntity {
    fn name(&self) -> String {
        match self {
            Self::Cargo(task) => task.name(),
            Self::Cp(task) => task.name(),
            Self::Env(task) => task.name(),
            Self::Link(task) => task.name(),
            Self::Sh(task) => task.name(),
            Self::Wget(task) => task.name(),
            Self::Brew(task) => task.name(),
        }
    }

    async fn execute(&self, ctx: &TaskContext) -> TaskResult {
        match self {
            Self::Cargo(task) => task.execute(ctx).await,
            Self::Cp(task) => task.execute(ctx).await,
            Self::Env(task) => task.execute(ctx).await,
            Self::Link(task) => task.execute(ctx).await,
            Self::Sh(task) => task.execute(ctx).await,
            Self::Wget(task) => task.execute(ctx).await,
            Self::Brew(task) => task.execute(ctx).await,
        }
    }
}

pub type TaskGroups = HashMap<String, Vec<(String, TaskEntity)>>;
pub type ScheduledTasks<'a> = Vec<(&'a str, &'a [(String, TaskEntity)])>;

/// Compiled configuration
pub struct PlayBook {
    taskgroups: TaskGroups,
    base: PathBuf,
    task_ids: Vec<String>,
    serialize_ids: Vec<String>,
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
pub struct TaskContext<'a> {
    /// Base directory to execute deploy
    pub base: PathBuf,
    /// Dry-run flag
    pub dryrun: bool,
    /// Selected deploy scenario
    pub scenario: String,
    /// Cache shared between same task type
    pub cache: &'a RwLock<Option<Vec<u8>>>,
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
    /// Failed to load cache
    CannotLoadCache(String),
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

fn parse_task<T: TaskBuilder>(yaml: &ast::Value) -> Result<(String, TaskEntity), Error> {
    let obj = yaml
        .as_hash()
        .ok_or_else(|| Error::InvalidPlaybook("task must be hash".to_owned(), yaml.to_owned()))?;
    if let Some(ast::Value::Str(key)) = obj.get("type") {
        if let Some(task) = T::parse(key.as_str(), obj) {
            Ok((key.to_owned(), task?))
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

fn parse_taskgroups<T: TaskBuilder>(yaml: &ast::Value) -> Result<TaskGroups, Error> {
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
                    .map(|src| parse_task::<T>(src))
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
    scenarios.iter().find(|&scenario| {
        scenario.matches.iter().all(|matcher| match matcher {
            TargetMatcher::HostName(_, hostname_re) => {
                hostname_re.is_match(&node_info.hostname.to_string_lossy())
            }
            TargetMatcher::Root(is_root) => *is_root == node_info.root,
        })
    })
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

pub trait TaskBuilder {
    fn parse(key: &str, hash: &HashMap<String, ast::Value>) -> Option<Result<TaskEntity, Error>>;
    fn ids(&self) -> &[&str];
    fn serialize_ids(&self) -> &[&str];
    fn cache(&self, key: &str) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerboseLevel {
    Compact,
    ShowAllTask,
}

#[cfg(not(feature = "jsonnet"))]
fn load_config_str(config: &str) -> Result<String, Error> {
    fs::read_to_string(Path::new(&config)).map_err(|e| {
        Error::PlaybookLoadFailed(format!("cannot read playbook {} due to {:?}", config, e))
    })
}

#[cfg(feature = "jsonnet")]
fn load_config_str(config: &str) -> Result<String, Error> {
    if config.ends_with(".jsonnet") {
        let mut vm = jsonnet::JsonnetVm::new();
        let result = vm.evaluate_file(config).map_err(|e| {
            Error::PlaybookLoadFailed(format!("cannot load playbook {} due to {:?}", config, e))
        })?;
        Ok(result.as_str().to_owned())
    } else {
        fs::read_to_string(Path::new(&config)).map_err(|e| {
            Error::PlaybookLoadFailed(format!("cannot read playbook {} due to {:?}", config, e))
        })
    }
}

impl PlayBook {
    /// Load configuration from yaml text with taskbuilders.
    pub fn load_config<T: TaskBuilder>(config: &str, taskbuilders: &T) -> Result<Self, Error> {
        let playbook_src = load_config_str(config)?;
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
        let taskgroups = parse_taskgroups::<T>(taskgroups)?;
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
                .ids()
                .iter()
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>(),
            base: Path::new(config)
                .parent()
                .ok_or_else(|| {
                    Error::PlaybookLoadFailed(format!("cannot take parent of {}", config))
                })?
                .to_owned(),
            serialize_ids: taskbuilders
                .serialize_ids()
                .iter()
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>(),
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
    pub async fn execute_graphicaly(
        &self,
        dryrun: bool,
        scenario: Option<&str>,
        verbose_level: &VerboseLevel,
    ) -> Result<HashMap<String, Vec<u8>>, Error> {
        let (scenario, taskgroups) = self.deploys(scenario)?;
        let mut caches = HashMap::new();
        for task in &self.task_ids {
            caches.insert(task, Arc::new(RwLock::new(None)));
        }
        let serialize_lock = Arc::new(
            self.serialize_ids
                .iter()
                .map(|id| (id.to_owned(), Mutex::new(())))
                .collect::<HashMap<_, _>>(),
        );

        let change_count = Arc::new(RwLock::new(0));
        let skip_count = Arc::new(RwLock::new(0));

        let tasks = taskgroups
            .iter()
            .map(|(group, tasks)| {
                tasks
                    .iter()
                    .map(|(id, task)| (*group, id, task))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .concat();
        futures::stream::iter(tasks)
            .for_each(|(group, id, task)| {
                let scenario = scenario.clone();
                let caches = caches.clone();
                let change_count = change_count.clone();
                let skip_count = skip_count.clone();
                let serialize_lock = serialize_lock.clone();
                async move {
                    let _guard = if let Some(lock) = serialize_lock.get(id) {
                        Some(lock.lock().await)
                    } else {
                        None
                    };
                    let task_name = task.name();
                    let ctx = TaskContext {
                        dryrun,
                        scenario: scenario.clone(),
                        base: self.base.clone(),
                        cache: caches.get(&id).expect("already registered"),
                    };
                    let result = task.execute(&ctx).await;
                    match (result, verbose_level) {
                        (Ok(true), VerboseLevel::Compact) => {
                            *change_count.write().await += 1;
                            println!("[{}]", group);
                            println!(
                                "{}[Changed] {}{}",
                                color::Fg(color::Yellow),
                                color::Fg(color::White),
                                task_name
                            );
                        }
                        (Ok(false), VerboseLevel::Compact) => {
                            *skip_count.write().await += 1;
                        }
                        (Ok(true), VerboseLevel::ShowAllTask) => {
                            println!("[{}]", group);
                            println!(
                                "{}[Changed] {}{}",
                                color::Fg(color::Yellow),
                                color::Fg(color::White),
                                task_name
                            );
                        }
                        (Ok(false), VerboseLevel::ShowAllTask) => {
                            println!("[{}]", group);
                            println!(
                                "{}[Ok]      {}{}",
                                color::Fg(color::Green),
                                color::Fg(color::LightWhite),
                                task_name
                            );
                        }
                        (Err(TaskError::WellKnown(msg)), _) => {
                            println!("[{}]", group);
                            println!(
                                "{}[Failed]  {}{}",
                                color::Fg(color::Red),
                                color::Fg(color::Reset),
                                task_name
                            );
                            println!("  -> {}", msg);
                        }
                        (Err(TaskError::Unknown(e)), _) => {
                            println!("[{}]", group);
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
            })
            .await;
        if verbose_level == &VerboseLevel::Compact {
            if *change_count.read().await > 0 {
                println!(
                    "{}[Changed] {}{} tasks",
                    color::Fg(color::Yellow),
                    color::Fg(color::White),
                    change_count.read().await
                );
            }
            if *skip_count.read().await > 0 {
                println!(
                    "{}[Ok] {}{} tasks",
                    color::Fg(color::Green),
                    color::Fg(color::White),
                    skip_count.read().await
                );
            }
        }
        Ok(futures::stream::iter(caches)
            .filter_map(|(k, v)| async move {
                v.read()
                    .await
                    .as_ref()
                    .map(|v| (k.to_owned(), v.to_owned()))
            })
            .collect::<HashMap<_, _>>()
            .await)
    }
}
