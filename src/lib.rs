use std::any::Any;
use std::cell::RefCell;
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

pub trait Task {
    fn name(&self) -> String;
    fn execute(&self, ctx: &TaskContext) -> TaskResult;
}

#[derive(Debug)]
enum TargetMatcher {
    HostName(regex::Regex),
    Root(bool),
}

#[derive(Debug)]
struct Scenario {
    name: String,
    tasks: Vec<String>,
    matches: Vec<TargetMatcher>,
}

pub type TaskGroups = HashMap<String, Vec<(String, Box<dyn Task>)>>;
pub type ScheduledTasks<'a> = Vec<(&'a str, &'a [(String, Box<dyn Task>)])>;

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

#[derive(Debug)]
pub struct TaskContext {
    pub base: PathBuf,
    pub dryrun: bool,
    pub scenario: String,
    pub cache: Rc<RefCell<Option<Box<dyn Any>>>>,
}

#[derive(Debug, Clone)]
pub enum Error {
    PlaybookLoadFailed(String),
    InvalidPlaybook(String, ast::Value),
    TaskGroupNotFound(String),
    AnyScenarioDoesNotMatch,
    CannotResolveVar(String, std::env::VarError),
}

type TaskResult = Result<bool, TaskError>;
#[derive(Error, Debug)]
pub enum TaskError {
    #[error("wellknown {0}")]
    WellKnown(String),
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
                Ok(TargetMatcher::HostName(hostname_regex))
            }
            "root" => Ok(TargetMatcher::Root(val.as_bool().ok_or_else(|| {
                Error::InvalidPlaybook("matcher.root must be boolean".to_owned(), yaml.to_owned())
            })?)),
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

fn match_scenario(scenarios: &[Scenario]) -> Option<&Scenario> {
    for scenario in scenarios {
        if scenario.matches.iter().all(|matcher| match matcher {
            TargetMatcher::HostName(hostname_re) => hostname::get()
                .map(|hostname| hostname_re.is_match(&hostname.to_string_lossy()))
                .unwrap_or(false),
            #[cfg(target_family = "unix")]
            TargetMatcher::Root(is_root) => unsafe { (libc::getuid() == 0) == *is_root },
        }) {
            return Some(scenario);
        }
    }
    None
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
    pub fn deploys(&self) -> Result<(String, ScheduledTasks), Error> {
        let scenario = match_scenario(&self.scenarios).ok_or(Error::AnyScenarioDoesNotMatch)?;
        Ok((
            scenario.name.to_owned(),
            enlist_taskgroups(&self.taskgroups, scenario.tasks.as_slice())?,
        ))
    }

    pub fn execute_graphicaly(&self, dryrun: bool) -> Result<(), Error> {
        let (scenario, taskgroups) = self.deploys()?;
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
