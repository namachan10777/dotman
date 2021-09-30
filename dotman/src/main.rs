use std::fmt::format;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use std::{fs, io, process};
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

impl Task {
    fn name(&self) -> String {
        match self {
            Task::Cp {
                src,
                dest,
                merge: _,
            } => format!("cp {} => {}", src, dest),
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
struct PlayBook {
    taskgroups: HashMap<String, Vec<Task>>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone)]
enum Error {
    FailedToLoadPlaybook,
    TaskGroupNotFound(String),
    AnyScenarioDoesNotMatch,
}

#[derive(Debug)]
enum TaskResult {
    Changed,
    Failed,
    Success,
}

#[derive(Debug, Clone)]
enum FileType {
    Symlink(PathBuf),
    File(PathBuf),
    Other(PathBuf),
    Nothing,
}

fn enlist_descendants(path: &Path) -> io::Result<Vec<PathBuf>> {
    if fs::metadata(path)?.is_dir() {
        Ok(fs::read_dir(path)?
            .into_iter()
            .map(|entry| enlist_descendants(&entry?.path()))
            .collect::<io::Result<Vec<_>>>()?
            .concat())
    } else {
        Ok(vec![path.to_owned()])
    }
}

fn file_table(src: &str, dest: &str) -> io::Result<HashMap<PathBuf, (FileType, FileType)>> {
    let src_descendants = enlist_descendants(Path::new(src))?;
    let dest_descendants = enlist_descendants(Path::new(dest))?;
    let mut hash = HashMap::new();
    for src_descendant in src_descendants {
        let meta = fs::metadata(&src_descendant)?;
        let src_filetype = if meta.is_file() {
            FileType::File(src_descendant.to_owned())
        } else {
            FileType::Other(src_descendant.to_owned())
        };
        hash.insert(
            src_descendant
                .strip_prefix(&Path::new(src))
                .unwrap()
                .to_owned(),
            (src_filetype, FileType::Nothing),
        );
    }
    for dest_descendant in dest_descendants {
        let meta = fs::metadata(&dest_descendant)?;
        let dest_filetype = if meta.is_file() {
            FileType::File(dest_descendant.to_owned())
        } else {
            FileType::Other(dest_descendant.to_owned())
        };
        hash.entry(
            dest_descendant
                .strip_prefix(&Path::new(src))
                .unwrap()
                .to_owned(),
        )
        .and_modify(|pair| *pair = (pair.0.clone(), dest_filetype.clone()))
        .or_insert((FileType::Nothing, dest_filetype));
    }
    Ok(hash)
}

fn execute_cp(src: &str, dest: &str, merge: bool) -> TaskResult {
    TaskResult::Failed
}

fn execute(task: &Task, dryrun: bool) -> TaskResult {
    match task {
        Task::Cp { src, dest, merge } => execute_cp(src, dest, *merge),
    }
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

type Stats = Vec<(String, Vec<(String, TaskResult)>)>;

fn execute_deploy(playbook: &PlayBook, dryrun: bool) -> Result<Stats, Error> {
    let scenario = match_scenario(&playbook.scenarios).ok_or(Error::AnyScenarioDoesNotMatch)?;
    let taskgroups = enlist_taskgroups(&playbook.taskgroups, scenario.tasks.as_slice())?;
    Ok(taskgroups
        .iter()
        .map(|(name, tasks)| {
            (
                name.to_owned().to_owned(),
                tasks
                    .iter()
                    .map(|task| (task.name(), execute(task, dryrun)))
                    .collect::<Vec<(String, TaskResult)>>(),
            )
        })
        .collect::<Stats>())
}

fn run(opts: Opts) -> Result<(), Error> {
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let playbook = load_config(opts.config).unwrap();
            let result = execute_deploy(&playbook, false)?;
            println!("{:#?}", result);
            Ok(())
        }
        Subcommand::DryRun(opts) => {
            let playbook = load_config(opts.config).unwrap();
            let result = execute_deploy(&playbook, true)?;
            println!("{:#?}", result);
            Ok(())
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match run(opts) {
        Ok(()) => (),
        Err(Error::AnyScenarioDoesNotMatch) => {
            eprintln!("any scenario does not match");
            process::exit(-1);
        }
        Err(Error::TaskGroupNotFound(taskgroup_name)) => {
            eprintln!("taskgroup \"{}\" does not found", taskgroup_name);
            process::exit(-1);
        }
        Err(Error::FailedToLoadPlaybook) => {
            eprintln!("failed to load playbook");
            process::exit(-1);
        }
    }
}
