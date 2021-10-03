use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use std::{env, fs, io, path, process};
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
enum TemplateValue {
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug)]
enum Task {
    Cp {
        src: String,
        dest: String,
        merge: bool,
        templates: HashMap<Vec<String>, HashMap<String, TemplateValue>>,
    },
}

impl Task {
    fn name(&self) -> String {
        match self {
            Task::Cp {
                src,
                dest,
                merge: _,
                templates: _,
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

#[derive(Debug)]
struct Context {
    base: PathBuf,
}

#[derive(Debug, Clone)]
enum Error {
    FailedToLoadPlaybook,
    TaskGroupNotFound(String),
    AnyScenarioDoesNotMatch,
    CannotResolveVar(String),
}

#[derive(Debug)]
enum TaskResult {
    Changed,
    Failed,
    Success,
}

#[derive(Debug, Clone)]
enum FileType {
    #[allow(dead_code)]
    Symlink(PathBuf),
    File(PathBuf),
    Other(PathBuf),
    Nothing(PathBuf),
    Dir(PathBuf),
}

fn resolve_desitination_path(path: &str) -> Result<PathBuf, Error> {
    Ok(Path::new(
        &path
            .split(path::MAIN_SEPARATOR)
            .map(|elem| {
                if let Some(var_name) = elem.strip_prefix('$') {
                    env::var(var_name).map_err(|_| Error::CannotResolveVar(elem.to_owned()))
                } else if elem.starts_with("\\$") {
                    Ok(elem[1..].to_owned())
                } else {
                    Ok(elem.to_owned())
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(&path::MAIN_SEPARATOR.to_string()),
    )
    .to_owned())
}

fn enlist_descendants(path: &Path) -> io::Result<Vec<PathBuf>> {
    if fs::metadata(path)?.is_dir() {
        let mut entries = fs::read_dir(path)?
            .into_iter()
            .map(|entry| enlist_descendants(&entry?.path()))
            .collect::<io::Result<Vec<_>>>()?
            .concat();
        entries.push(path.to_owned());
        Ok(entries)
    } else {
        Ok(vec![path.to_owned()])
    }
}

fn file_table(src: &Path, dest: &Path) -> io::Result<HashMap<PathBuf, (FileType, FileType)>> {
    let src_descendants = enlist_descendants(src)?;
    let dest_descendants = enlist_descendants(dest)?;
    let mut hash = HashMap::new();
    for src_descendant in src_descendants {
        let meta = fs::metadata(&src_descendant)?;
        let src_filetype = if meta.is_file() {
            FileType::File(src_descendant.to_owned())
        } else if meta.is_dir() {
            FileType::Dir(src_descendant.to_owned())
        } else {
            FileType::Other(src_descendant.to_owned())
        };
        hash.insert(
            src_descendant
                .strip_prefix(&Path::new(src))
                .unwrap()
                .to_owned(),
            (
                src_filetype,
                FileType::Nothing(dest.join(src_descendant.strip_prefix(src).unwrap())),
            ),
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
                .strip_prefix(&Path::new(dest))
                .unwrap()
                .to_owned(),
        )
        .and_modify(|pair| *pair = (pair.0.clone(), dest_filetype.clone()))
        .or_insert((
            FileType::Nothing(src.join(dest_descendant.strip_prefix(dest).unwrap())),
            dest_filetype,
        ));
    }
    Ok(hash)
}

type Templates = HashMap<Vec<String>, HashMap<String, TemplateValue>>;

fn match_template_target<'a>(
    templates: &'a Templates,
    src: &Path,
    target: &Path,
) -> Option<&'a HashMap<String, TemplateValue>> {
    for (target_patterns, var_set) in templates {
        let target = target.strip_prefix(src).unwrap_or(target);
        for pattern in target_patterns {
            if Path::new(pattern) == target {
                return Some(var_set);
            }
        }
    }
    None
}

enum SyncError {
    UnhandledIoError(io::Error),
    Failed(String),
}

impl From<io::Error> for SyncError {
    fn from(e: io::Error) -> Self {
        SyncError::UnhandledIoError(e)
    }
}

fn sync_file(
    src: &FileType,
    dest: &FileType,
    dryrun: bool,
    merge: bool,
    _templates: &Templates,
    _src_base: &Path,
) -> Result<bool, SyncError> {
    match (&src, &dest, merge) {
        (FileType::Nothing(_), FileType::Nothing(_), _) => Ok(false),
        (FileType::Nothing(_), _, true) => Ok(false),
        (FileType::Nothing(_), FileType::Dir(dest), false) => {
            // TODO: fix to unlink
            if !dryrun {
                fs::remove_dir(dest)?;
            }
            Ok(true)
        }
        (FileType::Nothing(_), FileType::Symlink(dest), false) => {
            // TODO: fix to unlink
            if !dryrun {
                fs::remove_file(dest)?;
            }
            Ok(true)
        }
        (FileType::Nothing(_), FileType::File(dest) | FileType::Other(dest), false) => {
            if !dryrun {
                fs::remove_file(dest)?;
            }
            Ok(true)
        }
        (&FileType::File(src), &FileType::File(dest), _) => {
            let src_buf = fs::read(src)?;
            let dest_buf = fs::read(dest)?;
            if src_buf != dest_buf {
                if !dryrun {
                    fs::copy(src, dest)?;
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        (&FileType::File(src), &FileType::Dir(dest), _) => {
            if !dryrun {
                fs::remove_dir(dest)?;
                fs::copy(src, dest)?;
            }
            Ok(true)
        }
        (&FileType::File(src), &FileType::Other(dest), _) => {
            if !dryrun {
                fs::remove_file(dest)?;
                fs::copy(src, dest)?;
            }
            Ok(true)
        }
        (&FileType::File(src), &FileType::Nothing(dest), _) => {
            if !dryrun {
                fs::create_dir_all(dest.parent().unwrap())?;
                fs::copy(src, dest)?;
            }
            Ok(true)
        }
        (&FileType::File(_), &FileType::Symlink(_), _) => {
            Err(SyncError::Failed("symlink is unsupported.".to_owned()))
        }
        (FileType::Dir(_), _, _) => Ok(false),
        (FileType::Other(_), _, _) => Err(SyncError::Failed("unknown file type".to_owned())),
        (FileType::Symlink(_), _, _) => {
            Err(SyncError::Failed("symlink is unsupported.".to_owned()))
        }
    }
}

// TODO: handle error when src directory is not found.
fn execute_cp(
    ctx: &Context,
    src: &str,
    dest: &str,
    merge: bool,
    dryrun: bool,
    templates: &HashMap<Vec<String>, HashMap<String, TemplateValue>>,
) -> TaskResult {
    let src_base = ctx.base.join(Path::new(src));
    if let Ok(dest) = resolve_desitination_path(dest) {
        if let Ok(tbl) = file_table(&src_base, &dest) {
            let mut changed = false;
            for (src, dest) in tbl.values() {
                match sync_file(src, dest, dryrun, merge, templates, &src_base) {
                    Ok(change_flag) => {
                        changed |= change_flag;
                    }
                    Err(_) => {
                        return TaskResult::Failed;
                    }
                }
            }
            if changed {
                return TaskResult::Changed;
            } else {
                return TaskResult::Success;
            }
        }
    }
    TaskResult::Failed
}

fn execute(ctx: &Context, task: &Task, dryrun: bool) -> TaskResult {
    match task {
        Task::Cp {
            src,
            dest,
            merge,
            templates,
        } => execute_cp(ctx, src, dest, *merge, dryrun, templates),
    }
}

fn parse_cp_templates(yaml: &Yaml) -> Result<(Vec<String>, HashMap<String, TemplateValue>), Error> {
    let hash = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    let target = match hash
        .get(&Yaml::String("target".to_owned()))
        .ok_or(Error::FailedToLoadPlaybook)?
    {
        Yaml::Array(targets) => targets
            .iter()
            .map(|target| {
                target
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or(Error::FailedToLoadPlaybook)
            })
            .collect::<Result<Vec<String>, Error>>(),
        Yaml::String(target) => Ok(vec![target.to_owned()]),
        _ => Err(Error::FailedToLoadPlaybook),
    }?;
    let variables = hash
        .get(&Yaml::String("vars".to_owned()))
        .ok_or(Error::FailedToLoadPlaybook)?
        .as_hash()
        .ok_or(Error::FailedToLoadPlaybook)?
        .into_iter()
        .map(|(name, val)| {
            let name = name.as_str().ok_or(Error::FailedToLoadPlaybook)?.to_owned();
            let val = match val {
                Yaml::String(str) => TemplateValue::String(str.to_owned()),
                Yaml::Integer(int) => TemplateValue::Int(*int),
                Yaml::Real(float) => TemplateValue::Float(float.parse().unwrap()),
                _ => return Err(Error::FailedToLoadPlaybook),
            };
            Ok((name, val))
        })
        .collect::<Result<HashMap<String, TemplateValue>, Error>>()?;
    Ok((target, variables))
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
                let templates = obj
                    .get(&Yaml::String("templates".to_owned()))
                    .map(|templates| {
                        templates.as_vec().ok_or(Error::FailedToLoadPlaybook)?.iter().map(parse_cp_templates).collect::<Result<HashMap<Vec<String>, HashMap<String, TemplateValue>>, Error>>()
                    })
                    .unwrap_or_else(|| Ok(HashMap::new()))?;
                Ok(Task::Cp {
                    src,
                    dest,
                    merge,
                    templates,
                })
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

fn load_config(config: &str) -> Result<PlayBook, Error> {
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

fn execute_deploy(ctx: &Context, playbook: &PlayBook, dryrun: bool) -> Result<Stats, Error> {
    let scenario = match_scenario(&playbook.scenarios).ok_or(Error::AnyScenarioDoesNotMatch)?;
    let taskgroups = enlist_taskgroups(&playbook.taskgroups, scenario.tasks.as_slice())?;
    Ok(taskgroups
        .iter()
        .map(|(name, tasks)| {
            (
                name.to_owned().to_owned(),
                tasks
                    .iter()
                    .map(|task| (task.name(), execute(ctx, task, dryrun)))
                    .collect::<Vec<(String, TaskResult)>>(),
            )
        })
        .collect::<Stats>())
}

fn run(opts: Opts) -> Result<(), Error> {
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let playbook = load_config(&opts.config).unwrap();
            let ctx = Context {
                base: Path::new(&opts.config).parent().unwrap().to_owned(),
            };
            let result = execute_deploy(&ctx, &playbook, false)?;
            println!("{:#?}", result);
            Ok(())
        }
        Subcommand::DryRun(opts) => {
            let playbook = load_config(&opts.config).unwrap();
            let ctx = Context {
                base: Path::new(&opts.config).parent().unwrap().to_owned(),
            };
            let result = execute_deploy(&ctx, &playbook, true)?;
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
        Err(Error::CannotResolveVar(var)) => {
            eprintln!("cannot resolve var ${}", var);
            process::exit(-1);
        }
    }
}
