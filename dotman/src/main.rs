use std::fs;
use std::{collections::HashMap, path::Path};
use yaml_rust::{Yaml, YamlLoader};

use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    Deploy(DeployOpts),
    DryRun(DryRunOpts),
}

#[derive(Clap)]
struct DeployOpts {
    #[clap(short, long)]
    config: String,
}

#[derive(Clap)]
struct DryRunOpts {
    #[clap(short, long)]
    config: String,
}

#[derive(Debug)]
enum Task {
    Cp {
        src: String,
        dest: String,
        merge: bool,
    },
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
struct PlayBook {
    taskgroups: HashMap<String, Vec<Task>>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone)]
enum Error {
    FailedToLoadPlaybook,
}

fn parse_task(yaml: &Yaml) -> Result<Task, Error> {
    let obj = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    if let Some(Yaml::String(key)) = obj.get(&Yaml::String("type".to_owned())) {
        match key.as_str() {
            "cp" => {
                let src = obj
                    .get(&Yaml::String("src".to_owned()))
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .as_str()
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .to_owned();
                let dest = obj
                    .get(&Yaml::String("dest".to_owned()))
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .as_str()
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .to_owned();
                let merge = obj
                    .get(&Yaml::String("merge".to_owned()))
                    .map(|val| val.as_bool().ok_or(Error::FailedToLoadPlaybook))
                    .unwrap_or(Ok(true))?;
                Ok(Task::Cp { src, dest, merge })
            }
            _ => Err(Error::FailedToLoadPlaybook),
        }
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn parse_taskgroups(yaml: &Yaml) -> Result<HashMap<String, Vec<Task>>, Error> {
    yaml.as_hash()
        .ok_or(Error::FailedToLoadPlaybook)?
        .iter()
        .map(|(name, tasks)| match (name, tasks) {
            (Yaml::String(name), Yaml::Array(tasks)) => Ok((
                name.to_owned(),
                tasks
                    .iter()
                    .map(parse_task)
                    .collect::<Result<Vec<Task>, Error>>()?,
            )),
            _ => Err(Error::FailedToLoadPlaybook),
        })
        .collect::<Result<HashMap<String, Vec<Task>>, Error>>()
}

fn parse_matcher(yaml: &Yaml) -> Result<TargetMatcher, Error> {
    let obj = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    if let Some((Yaml::String(target), val)) = obj.iter().next() {
        match target.as_str() {
            "hostname" => {
                let hostname_regex =
                    regex::Regex::new(val.as_str().ok_or(Error::FailedToLoadPlaybook)?)
                        .map_err(|_| Error::FailedToLoadPlaybook)?;
                Ok(TargetMatcher::HostName(hostname_regex))
            }
            _ => Err(Error::FailedToLoadPlaybook),
        }
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn parse_scenario(yaml: &Yaml) -> Result<Scenario, Error> {
    let obj = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
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
                taskname
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or(Error::FailedToLoadPlaybook)
            })
            .collect::<Result<Vec<String>, Error>>()?;
        Ok(Scenario {
            tasks,
            matches,
            name: name.to_owned(),
        })
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn load_config(config: String) -> Result<PlayBook, Error> {
    let playbook_src =
        fs::read_to_string(Path::new(&config)).map_err(|_| Error::FailedToLoadPlaybook)?;
    let playbook_ast = YamlLoader::load_from_str(&playbook_src)
        .map_err(|_| Error::FailedToLoadPlaybook)?
        .get(0)
        .ok_or(Error::FailedToLoadPlaybook)?
        .as_hash()
        .ok_or(Error::FailedToLoadPlaybook)?
        .clone();
    if let (Some(taskgroups), Some(scenarios)) = (
        playbook_ast.get(&Yaml::String("taskgroups".to_owned())),
        playbook_ast.get(&Yaml::String("scenarios".to_owned())),
    ) {
        let taskgroups = parse_taskgroups(taskgroups)?;
        let scenarios = scenarios
            .as_vec()
            .ok_or(Error::FailedToLoadPlaybook)?
            .iter()
            .map(parse_scenario)
            .collect::<Result<Vec<Scenario>, Error>>()?;
        Ok(PlayBook {
            taskgroups,
            scenarios,
        })
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let _playbook = load_config(opts.config);
        }
        Subcommand::DryRun(opts) => {
            let _playbook = load_config(opts.config);
        }
    }
}
