use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use yaml_rust::{Yaml, YamlLoader};

mod tasks;

use thiserror::Error;

#[derive(Debug)]
enum Task {
    Cp(tasks::cp::CpTask),
}

trait TaskUnit {
    fn name(&self) -> String;
    fn execute(&self, ctx: &TaskContext) -> TaskResult;
}

impl Task {
    fn name(&self) -> String {
        match self {
            Task::Cp(cp) => cp.name(),
        }
    }
}

#[derive(Debug)]
enum TargetMatcher {
    HostName(regex::Regex),
}

#[derive(Debug)]
struct Scenario {
    name: String,
    tasks: Vec<String>,
    matches: Vec<TargetMatcher>,
}

#[derive(Debug)]
pub struct PlayBook {
    taskgroups: HashMap<String, Vec<Task>>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub base: PathBuf,
    pub dryrun: bool,
}

#[derive(Debug, Clone)]
pub enum Error {
    PlaybookLoadFailed(String),
    InvalidPlaybook(String, Yaml),
    TaskGroupNotFound(String),
    AnyScenarioDoesNotMatch,
    CannotResolveVar(String),
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

fn execute(ctx: &TaskContext, task: &Task) -> TaskResult {
    match task {
        Task::Cp(cp) => cp.execute(ctx),
    }
}

fn parse_task(yaml: &Yaml) -> Result<Task, Error> {
    let obj = yaml
        .as_hash()
        .ok_or_else(|| Error::InvalidPlaybook("task must be hash".to_owned(), yaml.to_owned()))?;
    if let Some(Yaml::String(key)) = obj.get(&Yaml::String("type".to_owned())) {
        match key.as_str() {
            "cp" => Ok(Task::Cp(tasks::cp::parse(obj)?)),
            taskname => Err(Error::InvalidPlaybook(
                format!("unsupported task \"{}\"", taskname),
                yaml.to_owned(),
            )),
        }
    } else {
        Err(Error::InvalidPlaybook(
            "task must have \"type\" property".to_owned(),
            yaml.to_owned(),
        ))
    }
}

fn parse_taskgroups(yaml: &Yaml) -> Result<HashMap<String, Vec<Task>>, Error> {
    yaml.as_hash()
        .ok_or_else(|| {
            Error::InvalidPlaybook("taskgroups must be hash".to_owned(), yaml.to_owned())
        })?
        .iter()
        .map(|(name, tasks)| match (name, tasks) {
            (Yaml::String(name), Yaml::Array(tasks)) => Ok((
                name.to_owned(),
                tasks
                    .iter()
                    .map(parse_task)
                    .collect::<Result<Vec<Task>, Error>>()?,
            )),
            _ => Err(Error::InvalidPlaybook(
                "children of taskgropus must be [string]: <task>[]".to_owned(),
                yaml.to_owned(),
            )),
        })
        .collect::<Result<HashMap<String, Vec<Task>>, Error>>()
}

fn parse_matcher(yaml: &Yaml) -> Result<TargetMatcher, Error> {
    let obj = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("matcher must be hash".to_owned(), yaml.to_owned())
    })?;
    if let Some((Yaml::String(target), val)) = obj.iter().next() {
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

fn parse_scenario(yaml: &Yaml) -> Result<Scenario, Error> {
    let obj = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("scenario mast be hash".to_owned(), yaml.to_owned())
    })?;
    if let (Some(Yaml::String(name)), Some(Yaml::Array(matchers)), Some(Yaml::Array(tasks))) = (
        obj.get(&Yaml::String("name".to_owned())),
        obj.get(&Yaml::String("match".to_owned())),
        obj.get(&Yaml::String("tasks".to_owned())),
    ) {
        let matches = matchers
            .iter()
            .map(parse_matcher)
            .collect::<Result<Vec<TargetMatcher>, Error>>()?;
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
        }) {
            return Some(scenario);
        }
    }
    None
}

fn enlist_taskgroups<'a>(
    taskgroups: &'a HashMap<String, Vec<Task>>,
    taskgroup_names: &'a [String],
) -> Result<Vec<(&'a str, &'a [Task])>, Error> {
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

impl PlayBook {
    pub fn load_config(config: &str) -> Result<Self, Error> {
        let playbook_src = fs::read_to_string(Path::new(&config)).map_err(|e| {
            Error::PlaybookLoadFailed(format!("cannot read playbook {} due to {:?}", config, e))
        })?;
        let playbook_ast = YamlLoader::load_from_str(&playbook_src)
            .map_err(|_| {
                Error::PlaybookLoadFailed(format!("playbook {} has invalid syntax", config))
            })?
            .get(0)
            .ok_or_else(|| Error::PlaybookLoadFailed(format!("playbook {} is empty", config)))?
            .as_hash()
            .ok_or_else(|| Error::PlaybookLoadFailed(format!("playbook {} is not a hash", config)))?
            .clone();
        let taskgroups = playbook_ast
            .get(&Yaml::String("taskgroups".to_owned()))
            .ok_or_else(|| Error::PlaybookLoadFailed("taskgroups is not found".to_owned()))?;
        let scenarios = playbook_ast
            .get(&Yaml::String("scenarios".to_owned()))
            .ok_or_else(|| Error::PlaybookLoadFailed("scenarios is not found".to_owned()))?;
        let taskgroups = parse_taskgroups(taskgroups)?;
        let scenarios = scenarios
            .as_vec()
            .ok_or_else(|| {
                Error::InvalidPlaybook("scenario must be array".to_owned(), scenarios.to_owned())
            })?
            .iter()
            .map(parse_scenario)
            .collect::<Result<Vec<Scenario>, Error>>()?;
        Ok(PlayBook {
            taskgroups,
            scenarios,
        })
    }
    pub fn execute_deploy(&self, ctx: &TaskContext) -> Result<Stats, Error> {
        let scenario = match_scenario(&self.scenarios).ok_or(Error::AnyScenarioDoesNotMatch)?;
        let taskgroups = enlist_taskgroups(&self.taskgroups, scenario.tasks.as_slice())?;
        Ok(taskgroups
            .iter()
            .map(|(name, tasks)| {
                (
                    name.to_owned().to_owned(),
                    tasks
                        .iter()
                        .map(|task| (task.name(), execute(ctx, task)))
                        .collect::<Vec<(String, TaskResult)>>(),
                )
            })
            .collect::<Stats>())
    }
}
